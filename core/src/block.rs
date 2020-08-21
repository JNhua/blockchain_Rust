use chrono::prelude::*;
use utils::coder;
use serde::{Deserialize, Serialize};
use crate::pow;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    pub time: i64,
    //transactions data merkle root hash
    pub tx_hash: [u8; 32],
    pub pre_hash: [u8; 32],
    //target bit
    pub bits: u32,
    //nonce
    pub nonce: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub hash: [u8; 32],
    //transactions data
    pub data: String,
}

impl Block {
    pub fn new_block(data: String, pre_hash: [u8; 32], bits: u32) -> Block {
        let transactions = coder::my_serialize(&data);
        let mut tx_hash = [0; 32];
        coder::get_hash(&transactions, &mut tx_hash);

        let mut block = Block {
            header: BlockHeader {
                time: Utc::now().timestamp(),
                tx_hash,
                pre_hash,
                bits,
                nonce: 0,
            },
            hash: [0; 32],
            data,
        };

        let pow = pow::ProofOfWork::new_proof_of_work(bits);
        pow.run(&mut block);
        block
    }
}