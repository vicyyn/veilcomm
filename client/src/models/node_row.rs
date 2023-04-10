#[derive(Clone, Debug, PartialEq)]
pub struct NodeRow {
    pub ip: String,
    pub port: u16,
    pub address: String,
}

impl NodeRow {
    pub fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 8000,
            address: "some address".to_string(),
        }
    }

    pub fn get_tuple(&self) -> (String, u16, String) {
        (self.ip.clone(), self.port, self.address.clone())
    }
}
