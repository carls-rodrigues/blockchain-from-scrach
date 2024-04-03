use crate::database::{block::Block, BlockFS};

use super::{
    genesis::Genesis, get_blocks_db_file_path, get_genesis_json_file_path,
    init_data_dir_if_not_exists, Account, Hash, Tx,
};
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
    latest_block: Option<Block>,
    latest_block_hash: Hash,
    has_genesis_block: bool,
}

impl State {
    pub fn new_state_from_disk(data_dir: &str) -> State {
        if let Err(err) = init_data_dir_if_not_exists(data_dir) {
            panic!("State: Error initializing data directory: {:?}", err);
        }
        let Ok(genesis_path) = get_genesis_json_file_path(data_dir) else {
            panic!("State: Error getting genesis file path");
        };
        let genesis = Genesis::load_genesis(&genesis_path);
        let mut balances = HashMap::new();
        for (account, balance) in genesis.get_balances().iter() {
            balances.insert(account.clone(), *balance);
        }
        let db_path = get_blocks_db_file_path(data_dir).unwrap();
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
            latest_block: None,
            has_genesis_block: false,
        };
        let Ok(cloned) = state.db_file.try_clone() else {
            panic!("Error cloning db file");
        };
        let scanner = std::io::BufReader::new(cloned);
        for line in scanner.lines() {
            if let Ok(block_fs) = serde_json::from_str::<BlockFS>(&line.unwrap()) {
                if state.apply_txs(block_fs.value.txs()).is_ok() {
                    state.latest_block = Some(block_fs.value.clone());
                    state.latest_block_hash = block_fs.key;
                    state.has_genesis_block = true;
                };
            }
        }
        state
    }

    pub fn apply_block(&mut self, block: &Block, state: &State) -> Result<(), String> {
        let next_expected_block_number = match state.latest_block {
            Some(ref block) => block.header().number() + 1,
            None => 0,
        };
        if state.has_genesis_block && block.header().number() != next_expected_block_number {
            return Err(format!(
                "next exptected number block must be {} not {}",
                next_expected_block_number,
                block.header().number()
            ));
        }
        let blocks_equals = state.latest_block == Some(block.clone());
        if state.has_genesis_block
            && state.latest_block.clone().unwrap().header().number() > 0
            && !blocks_equals
        {
            return Err(format!(
                "next block parent hash must be {:?} not {:?}",
                state.latest_block_hash,
                block.header().parent()
            ));
        }
        self.apply_txs(block.txs())?;
        Ok(())
    }
    fn apply_txs(&mut self, txs: &Vec<Tx>) -> Result<(), String> {
        for tx in txs {
            self.apply_tx(tx)?;
        }
        Ok(())
    }
    pub fn add_blocks(&mut self, blocks: Vec<Block>) -> Result<(), String> {
        for block in blocks {
            self.add_block(block)?;
        }
        Ok(())
    }
    pub fn add_block(&mut self, block: Block) -> Result<Hash, String> {
        let pending_state = self.copy();
        self.apply_block(&block, &pending_state)?;
        let block_hash = block.hash()?;
        let block_fs = BlockFS {
            key: block_hash,
            value: block,
        };
        let block_fs_json = serde_json::to_string(&block_fs).unwrap();
        println!("Persisting new block to disk");

        writeln!(self.db_file, "{}", block_fs_json).unwrap();
        self.balances = pending_state.balances;
        self.latest_block_hash = block_hash;
        self.latest_block = Some(block_fs.value);
        self.has_genesis_block = true;

        Ok(block_hash)
    }
    pub fn add_tx(&mut self, tx: &Tx) -> Result<(), String> {
        self.apply_tx(tx)?;
        self.tx_mempool.push(tx.clone());
        Ok(())
    }

    pub fn persist(&mut self) -> Result<Hash, String> {
        let block_height = match self.latest_block {
            Some(ref block) => block.header().number() + 1,
            None => 0,
        };
        let block = Block::new(
            self.latest_block_hash,
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self.tx_mempool.clone(),
            block_height,
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
    pub fn copy(&self) -> State {
        State {
            balances: self.balances.clone(),
            tx_mempool: self.tx_mempool.clone(),
            db_file: self.db_file.try_clone().unwrap(),
            latest_block: self.latest_block.clone(),
            latest_block_hash: self.latest_block_hash,
            has_genesis_block: self.has_genesis_block,
        }
    }
    pub fn get_balances(&self) -> &HashMap<Account, u64> {
        &self.balances
    }
    pub fn latest_block_hash(&self) -> Hash {
        self.latest_block_hash
    }
    pub fn latest_block(&self) -> &Option<Block> {
        &self.latest_block
    }
    pub fn apply_tx(&mut self, tx: &Tx) -> Result<(), String> {
        println!("Applying tx {:?}", tx);
        let from_balance = self.balances.get(tx.from()).unwrap_or(&0);
        let to_balance = self.balances.get(tx.to()).unwrap_or(&0);
        if tx.is_reward() {
            self.balances
                .insert(tx.to().clone(), to_balance + tx.value() as u64);
            return Ok(());
        }

        if tx.value() > self.balances[tx.from()] as i64 {
            let message = format!(
                "Wrong TX. Sender {} balance is {} TBB. Tx cost is {} TBB",
                tx.from(),
                self.balances[tx.from()],
                tx.value(),
            );
            return Err(message);
        }
        let from_balance = from_balance - tx.value() as u64;
        let to_balance = to_balance + tx.value() as u64;
        self.balances
            .insert(tx.from().clone(), from_balance - tx.value() as u64);
        self.balances
            .insert(tx.to().clone(), to_balance + tx.value() as u64);
        Ok(())
    }
}
