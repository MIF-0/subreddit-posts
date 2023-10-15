#[derive(Debug)]
pub struct Environment {
    pub application_id: String,
    pub application_secret: String,
    pub application_redirection_link: String,
    pub application_scope: String,
}

impl Environment {
    pub fn read_env() -> Environment {
        let application_id = Environment::read_env_property("APPLICATION_ID");
        let application_secret = Environment::read_env_property("APPLICATION_SECRET");
        let application_redirection_link =
            Environment::read_env_property("APPLICATION_REDIRECTION_LINK");
        let application_scope = Environment::read_env_property("APPLICATION_SCOPE");
        Environment {
            application_id,
            application_secret,
            application_redirection_link,
            application_scope,
        }
    }

    fn read_env_property<T>(name: &str) -> T
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        let value =
            std::env::var(name).unwrap_or_else(|_| panic!("Can't read property {} from env", name));

        value
            .parse()
            .unwrap_or_else(|_| panic!("Can't parse property {} of {}", value, name))
    }
}
