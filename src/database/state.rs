use crate::database::{block::Block, BlockFS};

use super::{genesis::Genesis, Account, Hash, Tx};
use data_encoding::HEXLOWER;
use std::{
    collections::HashMap,
    io::{BufRead, Write},
    time,
};

#[derive(Debug)]
pub struct State {
    balances: HashMap<Account, u64>,
    tx_mempool: Vec<Tx>,
    db_file: std::fs::File,
    latest_block_hash: Hash,
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
        let db_path = cwd.join("src/database/block.db");
        let db_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(db_path)
            .unwrap();

        let mut state = State {
            balances,
            tx_mempool: Vec::new(),
            db_file,
            latest_block_hash: [0; 32],
        };
        let Ok(cloned) = state.db_file.try_clone() else {
            panic!("Error cloning db file");
        };
        let scanner = std::io::BufReader::new(cloned);
        for line in scanner.lines() {
            let block_fs: BlockFS = serde_json::from_str(&line.unwrap()).unwrap();
            let Ok(_) = state.apply_block(block_fs.value) else {
                panic!("Error applying block");
            };
            state.latest_block_hash = block_fs.key;
        }
        state
    }

    pub fn apply_block(&mut self, block: Block) -> Result<(), String> {
        for tx in block.txs() {
            self.apply(tx.clone())?;
        }
        Ok(())
    }
    pub fn add_block(&mut self, block: Block) -> Result<(), String> {
        for tx in block.txs() {
            self.add_tx(tx)?
        }
        Ok(())
    }
    pub fn add_tx(&mut self, tx: &Tx) -> Result<(), String> {
        self.apply(tx.clone())?;
        self.tx_mempool.push(tx.clone());
        Ok(())
    }

    pub fn persist(&mut self) -> Result<Hash, String> {
        let block = Block::new(
            self.latest_block_hash,
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self.tx_mempool.clone(),
        );
        let block_hash = block.hash()?;
        let block_fs = BlockFS {
            key: block_hash,
            value: block,
        };
        let block_fs_json = serde_json::to_string(&block_fs).unwrap();
        println!("Persisting new block to disk");
        println!("Block created {:?}", HEXLOWER.encode(&block_hash));

        if self.db_file.write(block_fs_json.as_bytes()).is_err() {
            return Err("Error writing to file".to_string());
        }
        if self.db_file.write("\n".as_bytes()).is_err() {
            return Err("Error writing to file".to_string());
        }
        self.latest_block_hash = block_hash;
        self.tx_mempool.clear();

        Ok(self.latest_block_hash)
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
    pub fn latest_block_hash(&self) -> String {
        HEXLOWER.encode(&self.latest_block_hash)
    }
}
