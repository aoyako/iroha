//! This module contains implementations of smart-contract traits and instructions for [`Account`] structure
//! and implementations for account queries.

use iroha_data_model::{prelude::*, query::error::FindError};
use iroha_telemetry::metrics;

use super::prelude::*;

/// All instructions related to accounts:
/// - minting/burning public key into account signatories
/// - minting/burning signature condition check
/// - update metadata
/// - grant permissions and roles
/// - Revoke permissions or roles
pub mod isi {
    use iroha_data_model::isi::{
        error::{MintabilityError, RepetitionError},
        InstructionType,
    };

    use super::*;
    use crate::{role::RoleIdWithOwner, state::StateTransaction};

    impl Execute for Transfer<Account, AssetDefinitionId, Account> {
        fn execute(
            self,
            _authority: &AccountId,
            state_transaction: &mut StateTransaction<'_, '_>,
        ) -> Result<(), Error> {
            let Transfer {
                source,
                object,
                destination,
            } = self;

            let _ = state_transaction.world.account(&source)?;
            let _ = state_transaction.world.account(&destination)?;

            let asset_definition = state_transaction.world.asset_definition_mut(&object)?;

            if asset_definition.owned_by != source {
                return Err(Error::Find(FindError::Account(source)));
            }

            asset_definition.owned_by = destination.clone();
            state_transaction
                .world
                .emit_events(Some(AssetDefinitionEvent::OwnerChanged(
                    AssetDefinitionOwnerChanged {
                        asset_definition: object,
                        new_owner: destination,
                    },
                )));

            Ok(())
        }
    }

    impl Execute for SetKeyValue<Account> {
        #[metrics(+"set_account_key_value")]
        fn execute(
            self,
            _authority: &AccountId,
            state_transaction: &mut StateTransaction<'_, '_>,
        ) -> Result<(), Error> {
            let account_id = self.object;

            state_transaction
                .world
                .account_mut(&account_id)
                .map_err(Error::from)
                .map(|account| {
                    account
                        .metadata
                        .insert(self.key.clone(), self.value.clone())
                })?;

            state_transaction
                .world
                .emit_events(Some(AccountEvent::MetadataInserted(MetadataChanged {
                    target: account_id,
                    key: self.key,
                    value: self.value,
                })));

            Ok(())
        }
    }

    impl Execute for RemoveKeyValue<Account> {
        #[metrics(+"remove_account_key_value")]
        fn execute(
            self,
            _authority: &AccountId,
            state_transaction: &mut StateTransaction<'_, '_>,
        ) -> Result<(), Error> {
            let account_id = self.object;

            let value = state_transaction
                .world
                .account_mut(&account_id)
                .and_then(|account| {
                    account
                        .metadata
                        .remove(&self.key)
                        .ok_or_else(|| FindError::MetadataKey(self.key.clone()))
                })?;

            state_transaction
                .world
                .emit_events(Some(AccountEvent::MetadataRemoved(MetadataChanged {
                    target: account_id,
                    key: self.key,
                    value,
                })));

            Ok(())
        }
    }

    impl Execute for Grant<Permission, Account> {
        #[metrics(+"grant_account_permission")]
        fn execute(
            self,
            _authority: &AccountId,
            state_transaction: &mut StateTransaction<'_, '_>,
        ) -> Result<(), Error> {
            let account_id = self.destination;
            let permission = self.object;

            // Check if account exists
            state_transaction.world.account_mut(&account_id)?;

            if state_transaction
                .world
                .account_contains_inherent_permission(&account_id, &permission)
            {
                return Err(RepetitionError {
                    instruction: InstructionType::Grant,
                    id: permission.into(),
                }
                .into());
            }

            state_transaction
                .world
                .add_account_permission(&account_id, permission.clone());

            state_transaction
                .world
                .emit_events(Some(AccountEvent::PermissionAdded(
                    AccountPermissionChanged {
                        account: account_id,
                        permission,
                    },
                )));

            Ok(())
        }
    }

    impl Execute for Revoke<Permission, Account> {
        #[metrics(+"revoke_account_permission")]
        fn execute(
            self,
            _authority: &AccountId,
            state_transaction: &mut StateTransaction<'_, '_>,
        ) -> Result<(), Error> {
            let account_id = self.destination;
            let permission = self.object;

            // Check if account exists
            state_transaction.world.account(&account_id)?;

            if !state_transaction
                .world
                .remove_account_permission(&account_id, &permission)
            {
                return Err(FindError::Permission(permission.into()).into());
            }

            state_transaction
                .world
                .emit_events(Some(AccountEvent::PermissionRemoved(
                    AccountPermissionChanged {
                        account: account_id,
                        permission,
                    },
                )));

            Ok(())
        }
    }

    impl Execute for Grant<RoleId, Account> {
        #[metrics(+"grant_account_role")]
        fn execute(
            self,
            _authority: &AccountId,
            state_transaction: &mut StateTransaction<'_, '_>,
        ) -> Result<(), Error> {
            let account_id = self.destination;
            let role_id = self.object;

            state_transaction.world.role(&role_id)?;
            state_transaction.world.account(&account_id)?;

            if state_transaction
                .world
                .account_roles
                .insert(
                    RoleIdWithOwner::new(account_id.clone(), role_id.clone()),
                    (),
                )
                .is_some()
            {
                return Err(RepetitionError {
                    instruction: InstructionType::Grant,
                    id: IdBox::RoleId(role_id),
                }
                .into());
            }

            state_transaction
                .world
                .emit_events(Some(AccountEvent::RoleGranted(AccountRoleChanged {
                    account: account_id.clone(),
                    role: role_id,
                })));

            Ok(())
        }
    }

    impl Execute for Revoke<RoleId, Account> {
        #[metrics(+"revoke_account_role")]
        fn execute(
            self,
            _authority: &AccountId,
            state_transaction: &mut StateTransaction<'_, '_>,
        ) -> Result<(), Error> {
            let account_id = self.destination;
            let role_id = self.object;

            if state_transaction
                .world
                .account_roles
                .remove(RoleIdWithOwner {
                    account: account_id.clone(),
                    id: role_id.clone(),
                })
                .is_none()
            {
                return Err(FindError::Role(role_id).into());
            }

            state_transaction
                .world
                .emit_events(Some(AccountEvent::RoleRevoked(AccountRoleChanged {
                    account: account_id.clone(),
                    role: role_id,
                })));

            Ok(())
        }
    }

    /// Stop minting on the [`AssetDefinition`] globally.
    ///
    /// # Errors
    /// If the [`AssetDefinition`] is not `Mintable::Once`.
    #[inline]
    pub fn forbid_minting(definition: &mut AssetDefinition) -> Result<(), MintabilityError> {
        if definition.mintable == Mintable::Once {
            definition.mintable = Mintable::Not;
            Ok(())
        } else {
            Err(MintabilityError::ForbidMintOnMintable)
        }
    }

    #[cfg(test)]
    mod test {
        use iroha_data_model::{prelude::AssetDefinition, ParseError};
        use iroha_test_samples::gen_account_in;

        use crate::smartcontracts::isi::Registrable as _;

        #[test]
        fn cannot_forbid_minting_on_asset_mintable_infinitely() -> Result<(), ParseError> {
            let (authority, _authority_keypair) = gen_account_in("wonderland");
            let mut definition = AssetDefinition::numeric("test#hello".parse()?).build(&authority);
            assert!(super::forbid_minting(&mut definition).is_err());
            Ok(())
        }
    }
}

/// Implementations for account queries.
pub mod query {

    use eyre::Result;
    use iroha_data_model::{
        account::Account,
        permission::Permission,
        query::{dsl::CompoundPredicate, error::QueryExecutionFail as Error},
    };

    use super::*;
    use crate::{smartcontracts::ValidQuery, state::StateReadOnly};

    impl ValidQuery for FindRolesByAccountId {
        #[metrics(+"find_roles_by_account_id")]
        fn execute(
            self,
            filter: CompoundPredicate<RoleId>,
            state_ro: &impl StateReadOnly,
        ) -> Result<impl Iterator<Item = RoleId>, Error> {
            let account_id = &self.id;
            state_ro.world().account(account_id)?;
            Ok(state_ro
                .world()
                .account_roles_iter(account_id)
                .filter(move |&role_id| filter.applies(role_id))
                .cloned())
        }
    }

    impl ValidQuery for FindPermissionsByAccountId {
        #[metrics(+"find_permissions_by_account_id")]
        fn execute(
            self,
            filter: CompoundPredicate<Permission>,
            state_ro: &impl StateReadOnly,
        ) -> Result<impl Iterator<Item = Permission>, Error> {
            let account_id = &self.id;
            Ok(state_ro
                .world()
                .account_permissions_iter(account_id)?
                .filter(move |&permission| filter.applies(permission))
                .cloned())
        }
    }

    impl ValidQuery for FindAccounts {
        #[metrics(+"find_accounts")]
        fn execute(
            self,
            filter: CompoundPredicate<Account>,
            state_ro: &impl StateReadOnly,
        ) -> Result<impl Iterator<Item = Account>, Error> {
            Ok(state_ro
                .world()
                .accounts_iter()
                .filter(move |account| filter.applies_to_entry(account))
                .map(|account| account.to_owned()))
        }
    }

    impl ValidQuery for FindAccountsWithAsset {
        #[metrics(+"find_accounts_with_asset")]
        fn execute(
            self,
            filter: CompoundPredicate<Account>,
            state_ro: &impl StateReadOnly,
        ) -> std::result::Result<impl Iterator<Item = Account>, Error> {
            let asset_definition_id = self.asset_definition.clone();
            iroha_logger::trace!(%asset_definition_id);

            Ok(state_ro
                .world()
                .accounts_iter()
                .filter(move |account| {
                    state_ro
                        .world()
                        .assets()
                        .get(&AssetId::new(
                            asset_definition_id.clone(),
                            account.id().clone(),
                        ))
                        .is_some()
                })
                .filter(move |account| filter.applies_to_entry(account))
                .map(|account| account.to_owned()))
        }
    }
}
