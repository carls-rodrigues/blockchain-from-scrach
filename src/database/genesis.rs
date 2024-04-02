use std::{
    collections::HashMap,
    io::{Read, Write},
};

use super::Account;

static GENESIS_JSON: &str = r#"
{
    "genesis_time": "2019-03-18T00:00:00.000000000Z",
    "chain_id": "the-blockchain-bar-ledger",
    "balances": {
        "andrej": 1000000
    }
}
"#;

pub fn write_genesis_to_disk(path: &str) {
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(GENESIS_JSON.as_bytes()).unwrap();
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GenesisJson {
    genesis_time: String,
    chain_id: String,
    balances: HashMap<Account, u64>,
}

#[derive(Debug)]
pub struct Genesis {
    balances: HashMap<Account, u64>,
}

impl Genesis {
    pub fn load_genesis(path: &str) -> Genesis {
        let Ok(mut file) = std::fs::File::open(path) else {
            panic!("Genesis file not found");
        };
        let mut data = String::new();
        let Ok(_) = file.read_to_string(&mut data) else {
            panic!("Error reading genesis file");
        };
        let json: GenesisJson = serde_json::from_str(&data).unwrap();
        let mut balances = HashMap::new();
        for (account, balance) in json.balances.iter() {
            balances.insert(account.clone(), balance.to_owned());
        }
        Genesis { balances }
    }

    pub fn get_balance(&self, account: &Account) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }

    pub fn set_balance(&mut self, account: Account, balance: u64) {
        self.balances.insert(account, balance);
    }

    pub fn get_balances(&self) -> &HashMap<Account, u64> {
        &self.balances
    }
}
