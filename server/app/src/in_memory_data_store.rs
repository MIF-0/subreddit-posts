use std::sync::{RwLock};
use log::info;
use crate::data_store::DataStore;

pub struct InMemoryDataStore {
    login_request_id: RwLock<Option<String>>,
    auth_token: RwLock<Option<String>>,
}

impl InMemoryDataStore {
    pub fn new() -> InMemoryDataStore {
        InMemoryDataStore {
            login_request_id: RwLock::new(None),
            auth_token: RwLock::new(None),
        }
    }
}

impl DataStore for InMemoryDataStore {
    fn store_login_request_id(&self, login_request_id: String) {
        self.login_request_id
            .write()
            .expect("can't obtain the lock to store login request id")
            .insert(login_request_id);
    }

    fn retrieve_login_request_id(&self) -> String {
        self.login_request_id
            .read()
            .expect("can't obtain the lock to retrieve login request id")
            .as_ref()
            .unwrap()
            .clone()
    }

    fn store_auth_token(&self, auth_token: String) {
        info!("Storing auth token {:?}", auth_token);

        self.auth_token
            .write()
            .expect("can't obtain the lock to store token")
            .insert(auth_token);
    }

    fn retrieve_auth_token(&self) -> String {
        self.auth_token
            .read()
            .expect("can't obtain the lock to retrieve token")
            .as_ref()
            .unwrap()
            .clone()
    }
}