use super::Tx;

pub type Hash = [u8; 32];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Block {
    header: BlockHeader,
    #[serde(rename = "payload")]
    tx: Vec<Tx>,
}

impl Block {
    pub fn new(parent: Hash, time: u64, tx: Vec<Tx>, number: u64) -> Self {
        Self {
            header: BlockHeader {
                parent,
                time,
                number,
            },
            tx,
        }
    }
    pub fn hash(&self) -> Result<Hash, String> {
        let Ok(block_json) = serde_json::to_string(&self) else {
            return Err("failed to serialize block".to_string());
        };
        let hash = ring::digest::digest(&ring::digest::SHA256, block_json.as_bytes());
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_ref());
        Ok(result)
    }

    pub fn header(&self) -> &BlockHeader {
        &self.header
    }
    pub fn txs(&self) -> &Vec<Tx> {
        &self.tx
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    parent: Hash,
    number: u64,
    time: u64,
}

impl BlockHeader {
    pub fn parent(&self) -> &Hash {
        &self.parent
    }
    pub fn number(&self) -> u64 {
        self.number
    }
    pub fn time(&self) -> u64 {
        self.time
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockFS {
    pub key: Hash,
    #[serde(rename = "block")]
    pub value: Block,
}
