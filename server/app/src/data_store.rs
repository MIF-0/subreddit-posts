pub trait DataStore {
    fn store_login_request_id(&self, login_request_id: String);

    fn retrieve_login_request_id(&self) -> String;

    fn store_auth_token(&self, auth_token: String);

    fn retrieve_auth_token(&self) -> String;
}