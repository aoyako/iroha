use iroha_client::{
    client::{self, Client},
    config::Config,
    data_model::query::predicate::{string, value, PredicateBox},
};

fn main() {
    let config = Config::load("../configs/swarm/client.toml").unwrap();

    let client = Client::new(config);

    let result = client
        .build_query(client::permission::permission_schema())
        .with_filter(PredicateBox::new(
            value::QueryOutputPredicate::Identifiable(string::StringPredicate::starts_with("xor_")),
        ))
        .execute()
        .unwrap();
}
