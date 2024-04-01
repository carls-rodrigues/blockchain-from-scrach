use super::{genesis::Genesis, Account, Tx};
use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::{
    collections::HashMap,
    io::{BufRead, Read, Seek, Write},
};

pub type Snapshot = [u8; 32];

#[derive(Debug)]
pub struct State {
    balances: HashMap<Account, u64>,
    tx_mempool: Vec<Tx>,
    db_file: std::fs::File,
    snapshot: Snapshot,
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
            snapshot: [0; 32],
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
        let Ok(_) = state.do_snapshot() else {
            panic!("Error creating snapshot");
        };
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

    pub fn persist(&mut self) -> Result<Snapshot, String> {
        let mut mempool = Vec::<Tx>::with_capacity(self.tx_mempool.len());
        std::mem::swap(&mut self.tx_mempool, &mut mempool);
        for tx in mempool {
            let Ok(tx_json) = serde_json::to_string(&tx) else {
                return Err("Error serializing tx".to_string());
            };
            println!("Persisting new TX to disk");

            let file = &mut self.db_file;
            if file.write(tx_json.as_bytes()).is_err() {
                return Err("Error writing to file".to_string());
            };
            if file.write(b"\n").is_err() {
                return Err("Error writing to file".to_string());
            };
            self.do_snapshot()?;

            self.tx_mempool.push(tx);
        }
        let snapshot = HEXUPPER.encode(&self.snapshot).to_lowercase();
        println!("Snapshot created {:?}", snapshot);
        Ok(self.snapshot)
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

        if *balance_from < tx.value() as u64 {
            return Err("Insufficient balance".to_string());
        }
        let new_balance_from = balance_from - tx.value() as u64;
        let new_balance_to = balance_to + tx.value() as u64;
        self.balances.insert(tx.from().clone(), new_balance_from);
        self.balances.insert(tx.to().clone(), new_balance_to);
        Ok(())
    }
    pub fn get_balances(&mut self) -> &HashMap<Account, u64> {
        &self.balances
    }
    pub fn do_snapshot(&mut self) -> Result<(), String> {
        let Ok(_) = self.db_file.seek(std::io::SeekFrom::Start(0)) else {
            return Err("Error seeking file".to_string());
        };
        let Ok(digest) = self.sha256_digest() else {
            return Err("Error creating digest".to_string());
        };
        let snapshot = digest.as_ref();
        self.snapshot.copy_from_slice(snapshot);
        Ok(())
    }
    pub fn latest_snapshot(&self) -> String {
        HEXUPPER.encode(&self.snapshot).to_lowercase()
    }
    fn sha256_digest(&self) -> std::io::Result<Digest> {
        let mut buf = Vec::<u8>::new();
        let mut txs_data = std::io::BufReader::new(&self.db_file);
        txs_data.read_to_end(&mut buf).unwrap();
        let mut context = Context::new(&SHA256);
        context.update(&buf);
        Ok(context.finish())
    }
}
