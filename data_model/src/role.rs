//! Structures, traits and impls related to `Role`s.

#[cfg(not(feature = "std"))]
use alloc::{collections::btree_set, format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::collections::btree_set;

use derive_more::{Constructor, Display, FromStr};
use getset::Getters;
use iroha_data_model_derive::IdOrdEqHash;
use iroha_ffi::FfiType;
use iroha_schema::IntoSchema;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{
    ffi::declare_item,
    permission::{Permissions, Token as PermissionToken},
    Identifiable, Name, Registered,
};

/// Collection of [`RoleId`](Id)s
pub type RoleIds = btree_set::BTreeSet<<Role as Identifiable>::Id>;

/// Identification of a role.
#[derive(
    Debug,
    Display,
    Constructor,
    FromStr,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Decode,
    Encode,
    Deserialize,
    Serialize,
    FfiType,
    IntoSchema,
)]
#[repr(transparent)]
#[serde(transparent)]
// SAFETY: RoleId has no trap representations in Name
#[ffi_type(unsafe {robust})]
pub struct Id {
    /// Role name, should be unique .
    pub name: Name,
}

declare_item! {
    /// Role is a tag for a set of permission tokens.
    #[derive(
        Debug,
        Display,
        Clone,
        IdOrdEqHash,
        Getters,
        Decode,
        Encode,
        Deserialize,
        Serialize,
        FfiType,
        IntoSchema,
    )]
    #[cfg_attr(all(feature = "ffi_export", not(feature = "ffi_import")), iroha_ffi::ffi_export)]
    #[cfg_attr(feature = "ffi_import", iroha_ffi::ffi_import)]
    #[display(fmt = "{id}")]
    #[getset(get = "pub")]
    pub struct Role {
        /// Unique name of the role.
        #[getset(skip)]
        id: Id,
        /// Permission tokens.
        #[getset(skip)]
        permissions: Permissions,
    }
}

#[cfg_attr(
    all(feature = "ffi_export", not(feature = "ffi_import")),
    iroha_ffi::ffi_export
)]
#[cfg_attr(feature = "ffi_import", iroha_ffi::ffi_import)]
impl Role {
    /// Constructor.
    #[inline]
    pub fn new(id: <Self as Identifiable>::Id) -> <Self as Registered>::With {
        NewRole::new(id)
    }

    /// Get an iterator over [`permissions`](PermissionToken) of the `Role`
    #[inline]
    pub fn permissions(&self) -> impl ExactSizeIterator<Item = &PermissionToken> {
        self.permissions.iter()
    }

    /// Remove permission tokens with specified id from `Role`
    pub fn remove_permissions(&mut self, definition_id: &crate::permission::token::Id) {
        self.permissions
            .retain(|token| token.definition_id() != definition_id);
    }
}

impl Registered for Role {
    type With = NewRole;
}

/// Builder for [`Role`]
#[derive(
    Debug,
    Display,
    Clone,
    IdOrdEqHash,
    Getters,
    Decode,
    Encode,
    Deserialize,
    Serialize,
    FfiType,
    IntoSchema,
)]
pub struct NewRole {
    #[id(transparent)]
    inner: Role,
}

#[cfg(feature = "mutable_api")]
impl crate::Registrable for NewRole {
    type Target = Role;

    #[must_use]
    #[inline]
    fn build(self) -> Self::Target {
        self.inner
    }
}

#[cfg_attr(
    all(feature = "ffi_export", not(feature = "ffi_import")),
    iroha_ffi::ffi_export
)]
#[cfg_attr(feature = "ffi_import", iroha_ffi::ffi_import)]
impl NewRole {
    /// Constructor
    #[must_use]
    #[inline]
    fn new(id: <Role as Identifiable>::Id) -> Self {
        Self {
            inner: Role {
                id,
                permissions: Permissions::new(),
            },
        }
    }

    /// Identification
    #[inline]
    pub(crate) fn id(&self) -> &<Role as Identifiable>::Id {
        &self.inner.id
    }

    /// Add permission to the [`Role`]
    #[must_use]
    #[inline]
    pub fn add_permission(mut self, perm: impl Into<PermissionToken>) -> Self {
        self.inner.permissions.insert(perm.into());
        self
    }
}

/// The prelude re-exports most commonly used traits, structs and macros from this module.
pub mod prelude {
    pub use super::{Id as RoleId, NewRole, Role};
}
