use chrono::prelude::*;
use utils::coder;
use serde::{Deserialize, Serialize};
use crate::transaction;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct BlockHeader {
    pub time: i64,
    //transactions data merkle root hash
    pub tx_hash: [u8; 32],
    pub pre_hash: [u8; 32],
    //target bit
    pub bits: u32,
    //nonce
    pub nonce: u32,
    //after all transaction, the merkle hash of all account state
    pub state_root: [u8; 32],
    pub height: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub hash: [u8; 32],
    //transactions data
    pub transactions: Vec<transaction::Transaction>,
}

impl Block {
    fn min(a: u64, b: u64) -> u64 {
        if a >= b {
            a
        } else {
            b
        }
    }

    fn make_merkle_hash(vtxs: &Vec<transaction::Transaction>) -> [u8; 32] {
        if vtxs.len() == 0 {
            return [0; 32];
        }

        let mut vec_merkle_tree: Vec<[u8; 32]> = Vec::new();
        for tx in vtxs.iter() {
            vec_merkle_tree.push(tx.hash);
        }

        let mut j: u64 = 0;
        let mut size = vec_merkle_tree.len();

        while size > 1 {
            let mut i: u64 = 0;
            let temp_size = size as u64;
            while i < temp_size {
                let i2 = Block::min(i + 1, temp_size - 1);
                let index1: usize = (j + i) as usize;
                let index2: usize = (j + i2) as usize;
                let merge: ([u8; 32], [u8; 32]) = (vec_merkle_tree[index1], vec_merkle_tree[index2]);
                let merge_serialize = coder::my_serialize(&merge);
                let mut merge_hash: [u8; 32] = [0; 32];
                coder::get_hash(&merge_serialize[..], &mut merge_hash);
                vec_merkle_tree.push(merge_hash);
                i += 2;
            }

            j += temp_size;
            size = (size + 1) / 2;
        }

        let mut merkle_hash: [u8; 32] = [0; 32];
        match vec_merkle_tree.pop() {
            None => println!("vec_merkle_tree is empty!"),
            Some(t) => merkle_hash = t,
        }
        merkle_hash
    }

    pub fn new_block_template(
        transactions: Vec<transaction::Transaction>,
        pre_hash: [u8; 32], bits: u32, height: u64,
    ) -> Block {
        let tx_hash: [u8; 32] = Block::make_merkle_hash(&transactions);

        Block {
            header: BlockHeader {
                time: Utc::now().timestamp(),
                tx_hash,
                pre_hash,
                bits,
                nonce: 0,
                state_root: [0; 32],
                height,
            },
            hash: [0; 32],
            transactions,
        }
    }
}