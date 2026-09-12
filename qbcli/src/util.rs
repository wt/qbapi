pub mod auth;
pub mod cli;

use clap::ValueEnum as _;

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum AuthEnvironment {
    Sandbox,
    Production,
}

impl AuthEnvironment {
    pub fn arg_string(&self) -> String {
        self.to_possible_value().unwrap().get_name().to_owned()
    }

    pub fn api_client(&self) -> qbapi::Client {
        match &self {
            AuthEnvironment::Sandbox => qbapi::Client::new(qbapi::Environment::Sandbox),
            AuthEnvironment::Production => qbapi::Client::new(qbapi::Environment::Production),
        }
    }
}
