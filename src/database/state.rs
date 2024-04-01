use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use super::{genesis::Genesis, Account, Tx};

#[derive(Debug)]
pub struct State {
    balances: HashMap<Account, u64>,
    tx_mempool: Vec<Tx>,
    db_file: std::fs::File,
}

impl State {
    pub fn new_state_from_disk() -> State {
        let Ok(cwd) = std::env::current_dir() else {
            panic!("Error getting current directory");
        };
        let p = cwd.join("src/database/genesis.json");
        let Some(gen_file_path) = p.to_str() else {
            panic!("Error getting genesis file path");
        };
        let genesis = Genesis::load_genesis(gen_file_path);
        let mut balances = HashMap::new();
        for (account, balance) in genesis.get_balances().iter() {
            balances.insert(account.clone(), *balance);
        }
        let db_path = cwd.join("src/database/tx.db");
        let db_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(db_path)
            .unwrap();

        let mut state = State {
            balances,
            tx_mempool: Vec::new(),
            db_file,
        };
        let Ok(cloned) = state.db_file.try_clone() else {
            panic!("Error cloning db file");
        };
        let scanner = std::io::BufReader::new(cloned);
        for line in scanner.lines() {
            let tx: Tx = serde_json::from_str(&line.unwrap()).unwrap();
            if let Err(err) = state.apply(tx) {
                println!("Error applying tx: {}", err);
            }
        }
        state
    }

    pub fn add_tx(&mut self, tx: Tx) -> Result<(), String> {
        if let Err(err) = self.apply(tx.clone()) {
            println!("Error adding tx to state: {}", err);
            return Err(err);
        }
        self.tx_mempool.push(tx);
        Ok(())
    }

    pub fn persist(&mut self) -> Result<(), String> {
        let mut mempool = Vec::<Tx>::with_capacity(self.tx_mempool.len());
        std::mem::swap(&mut self.tx_mempool, &mut mempool);
        for tx in mempool {
            let Ok(tx_json) = serde_json::to_string(&tx) else {
                panic!("Error serializing tx");
            };
            let file = &mut self.db_file;
            if file.write(tx_json.as_bytes()).is_err() {
                return Err("Error writing to file".to_string());
            };
            if file.write(b"\n").is_err() {
                return Err("Error writing to file".to_string());
            };

            self.tx_mempool.push(tx);
        }
        Ok(())
    }

    pub fn close(&self) {
        self.db_file.sync_all().unwrap();
    }

    pub fn apply(&mut self, tx: Tx) -> Result<(), String> {
        let balance_from = self.balances.get(tx.from()).unwrap_or(&0);
        let balance_to = self.balances.get(tx.to()).unwrap_or(&0);
        if tx.is_reward() {
            self.balances
                .insert(tx.to().clone(), balance_to + tx.value() as u64);
            return Ok(());
        }
        println!("balance_from: {}", balance_from);
        println!("balance_to: {}", balance_to);
        if *balance_from < tx.value() as u64 {
            return Err("Insufficient balance".to_string());
        }
        let new_balance_from = balance_from - tx.value() as u64;
        let new_balance_to = balance_to + tx.value() as u64;
        self.balances.insert(tx.from().clone(), new_balance_from);
        self.balances.insert(tx.to().clone(), new_balance_to);
        Ok(())
    }
    pub fn get_balances(&self) -> &HashMap<Account, u64> {
        &self.balances
    }
}
