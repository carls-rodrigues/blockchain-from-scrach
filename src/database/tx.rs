use serde::{Deserialize, Serialize};

pub type Account = String;

pub fn new_account(value: &str) -> Account {
    value.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tx {
    from: Account,
    to: Account,
    value: i64,
    data: String,
}

impl Tx {
    pub fn new(from: Account, to: Account, value: &u64, data: &str) -> Tx {
        Tx {
            from,
            to,
            value: *value as i64,
            data: data.to_string(),
        }
    }
    pub fn is_reward(&self) -> bool {
        self.data == "reward"
    }
    pub fn from(&self) -> &Account {
        &self.from
    }
    pub fn to(&self) -> &Account {
        &self.to
    }
    pub fn value(&self) -> i64 {
        self.value
    }
    pub fn data(&self) -> &String {
        &self.data
    }
}
