use data_encoding::HEXLOWER;

use super::Tx;

pub type Hash = [u8; 32];

fn marshal_text(hash: &Hash) -> String {
    HEXLOWER.encode(hash).to_ascii_lowercase()
}
fn unmarshal_text(data: &str) -> Hash {
    let mut result = [0u8; 32];
    let bytes = HEXLOWER.decode(data.as_bytes()).unwrap();
    result.copy_from_slice(&bytes);
    result
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Block {
    header: BlockHeader,
    #[serde(rename = "payload")]
    tx: Vec<Tx>,
}

impl Block {
    pub fn new(parent: Hash, time: u64, tx: Vec<Tx>) -> Self {
        Self {
            header: BlockHeader { parent, time },
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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BlockHeader {
    parent: Hash,
    time: u64,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BlockFS {
    pub key: Hash,
    #[serde(rename = "block")]
    pub value: Block,
}
