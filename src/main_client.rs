use std::sync::Arc;


pub struct MainClient {
    // Telegram api client
    pub tg_api: Arc<mobot::API>,
    pub admins: Vec<i64>
}


impl MainClient {
    pub fn new(mobot_client: Arc<mobot::API>, admins: Vec<i64>) -> Self {
        Self {
            tg_api: mobot_client,
            admins: admins
        }
    }
    pub fn get_admins(&self) -> &Vec<i64> {
        &self.admins
    }
}
