use std::env;

pub struct Db {
    pub url: String,
}

impl Db {
    pub fn new(_env: &str) -> Self {
        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://user:password@mariadb:3306/erp_db".to_string());

        Db {
            url,
        }
    }
}