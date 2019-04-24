extern crate time;
extern crate crypto;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate canteen;
extern crate urlparse;
extern crate requests;

use urlparse::urlparse;
use canteen::*;
use canteen::utils;
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use ::time::*;
use std::rc::Rc;
use requests::*;


struct Transaction {
    sender: String,
    recipient: String,
    amount: String,
}

struct Block {
    index: i8,
    timestamp: i64,
    transaction: Transaction,
    proof: i32,
    previous_hash: String,
}

struct Blockchain {
    current_transactions: Vec<Transaction>,
    chain: Vec<Block>,
    nodes: Vec<String>,
}

impl Block {
    fn dumps(&self) -> String {
        let mut str = String::new();
        str.push_str(&String::from(self.index));
        str.push_str(&String::from(self.timestamp));
        str.push_str(&self.transaction.sender);
        str.push_str(&self.transaction.recipient);
        str.push_str(&self.transaction.amount);
        str.push_str(&String::from(self.proof));
        str.push_str(&self.previous_hash);
        str
    }

    fn myhash(block: Block) -> String {
        // create a SHA3-256 object
        let mut hasher = Sha3::sha3_256();
        // write input message
        hasher.input_str(&block.dumps());
        // read hash hexdigest
        hasher.result_str()
    }
}

impl Blockchain {
    fn new() -> Blockchain {
        Blockchain {
            current_transactions: Vec::new(),
            chain: Vec::new(),
            nodes: Vec::new(),
        }
    }

    fn init(&mut self) {
        self.new_block(100, Some(String::from("1")));
    }

    fn register_node(&mut self, address: String) {
        /*
        增加新节点
        ：参数address:node的address. Eg. 'http://127.0.0.1:5000'
        */
        parsed_url = urlparse(address);
        self.nodes.push(parsed_url);
    }

    fn valid_chain(chain: Vec<Block>) -> bool {
        let mut last_block = &chain[0];
        let mut current_idx = 1;

        while current_idx < chain.len() {
            let block = &chain[current_idx];
            println!("{:?}", last_block);
            println!("{:?}", block);
            println!("\n----------------\n");
            if block.previous_hash != block.myhash(last_block) {
                return false;
            }

            if !valid_proof(last_block.proof, block.proof) {
                return false;
            }

            last_block = block;
            current_idx += 1;
        };
        true
    }

    fn resolve_conflicts(&self) -> bool {
        let neighbours = &self.nodes;
        let new_chain = vec![];
        let max_length = self.chain.len();

        for node in neighbours {
            let response = requests::get("http://" + String::from(node) + "/chain");
            match response {
                Ok(re) =>,
                Err(E) => panic!("error"),
            }
        }
        false
    }

    fn new_block(&mut self, proof: i32,
                 previous_hash: Option<String>) -> Block {
        self.current_transactions.clear();
        let new_block = Block {
            index: (self.chain.len() + 1) as i8,
            timestamp: timestamp(),
            transaction: self.transaction,
            proof,
            previous_hash: match previous_hash {
                None => {
                    myhash(self.chain.last())
                }
                Some(val) => val,
            },
        };
        self.chain.push(Rc::clone(&new_block));
        new_block
    }

    fn timestamp() -> i64 {
        let timespec = time::get_time();
        timespec.sec * 1000 + (timespec.nsec as f64 / 1000.0 / 1000.0) as i64
    }

    fn valid_proof(last_proof: i32, proof: i32) -> bool {
        // create a SHA3-256 object
        let mut hasher = Sha3::sha3_256();
        // write input message
        hasher.input_str(&(String::from(last_proof) + String::from(proof)));

        let guess_hash = hasher.result_str();
        return guess_hash[0..4] == "0000";
    }
}

fn main() {
    let mut blockchian = Blockchain::new();
    blockchian.init();
}