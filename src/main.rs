extern crate time;
extern crate crypto;
extern crate urlparse;
extern crate requests;
extern crate rustc_serialize;
extern crate uuid;
extern crate canteen;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use canteen::{Canteen, Request, Response, Method};
use canteen::utils;

use urlparse::urlparse;
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use ::time::get_time;
use std::rc::Rc;
use requests::*;
use rustc_serialize::json;
use uuid::Uuid;

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: i8,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
struct Block {
    index: i8,
    timestamp: i64,
    transaction: Vec<Transaction>,
    proof: i32,
    previous_hash: String,
}

struct Blockchain {
    current_transactions: Vec<Transaction>,
    chain: Vec<Block>,
    nodes: Vec<String>,
}

impl Block {
    /*
    fn dumps(&self) -> String {
        let mut str = String::new();
        str.push_str(&(json::encode(&self.index).unwrap()
        +&json::encode(&self.timestamp).unwrap()
            +&json::encode(&self.transaction).unwrap())
        );
        str.push_str(&json::encode(&self.proof).unwrap());
        str.push_str(&self.previous_hash);
        str
    }
    */

    pub fn myhash(block: &Block) -> String {
        // create a SHA3-256 object
        let mut hasher = Sha3::sha3_256();
        // write input message
        hasher.input_str(&json::encode(&block).unwrap());
        // read hash hexdigest
        hasher.result_str()
    }
}

//关联函数
impl Blockchain {
    fn new() -> Blockchain {
        Blockchain {
            current_transactions: Vec::new(),
            chain: Vec::new(),
            nodes: Vec::new(),
        }
    }

    fn valid_proof(&last_proof: &i32, &proof: &i32) -> bool {
        // create a SHA3-256 object
        let mut hasher = Sha3::sha3_256();
        // write input message
        hasher.input_str(&(json::encode(&last_proof).unwrap() + &json::encode(&proof).unwrap()));

        let guess_hash = hasher.result_str();
        return guess_hash[0..4] == String::from("0000");
    }

    fn valid_chain(chain: &Vec<Block>) -> bool {
        let mut last_block = &chain[0];
        let mut current_idx = 1;

        while current_idx < chain.len() {
            let block = &chain[current_idx];
            println!("{:#?}", last_block);
            println!("{:#?}", block);
            println!("\n----------------\n");
            if block.previous_hash != Block::myhash(last_block) {
                return false;
            }

            if !Blockchain::valid_proof(&last_block.proof, &block.proof) {
                return false;
            }

            last_block = block;
            current_idx += 1;
        };
        true
    }

    fn timestamp() -> i64 {
        let timespec = get_time();
        timespec.sec * 1000 + (timespec.nsec as f64 / 1000.0 / 1000.0) as i64
    }

    fn proof_of_work(last_proof: i32) -> i32 {
        let mut proof = 0;
        while Blockchain::valid_proof(&last_proof, &proof) == false {
            proof += 1;
        }
        proof
    }
}

//方法
impl Blockchain {
    fn init(&mut self) {
        self.new_block(100, Some(String::from("1")));
    }

    fn register_node(&mut self, address: String) {
        /*
        增加新节点
        ：参数address:node的address. Eg. 'http://127.0.0.1:5000'
        */
        let parsed_url = urlparse(address);
        self.nodes.push(parsed_url.netloc);
    }

    fn resolve_conflicts(&mut self) -> bool {
        let neighbours = &self.nodes;
        let mut new_chain = vec![];
        let mut max_length = self.chain.len();

        for node in neighbours {
            let response =
                requests::get("http://".to_string() + node + "/chain").unwrap();

            if response.status_code() == requests::StatusCode::Ok {
                let res_json = response.json().unwrap();
                let length = res_json["length"].as_usize().unwrap();
                let chain = json::decode(res_json["chain"].as_str().unwrap()).unwrap();

                if length > max_length && Blockchain::valid_chain(&chain) {
                    max_length = length;
                    new_chain = chain;
                }
            }
        }
        if !new_chain.is_empty() {
            self.chain = new_chain;
            return true;
        }
        false
    }

    fn new_block(&mut self, proof: i32,
                 previous_hash: Option<String>) -> Block {
        self.current_transactions.clear();
        let new_block = Block {
            index: (self.chain.len() + 1) as i8,
            timestamp: Blockchain::timestamp(),
            transaction: self.current_transactions.clone(),
            proof,
            previous_hash: match previous_hash {
                None => {
                    Block::myhash(self.chain.last().unwrap())
                }
                Some(val) => val,
            },
        };
        self.chain.push(new_block.clone());
        new_block
    }

    fn last_block(&self) -> &Block {
        return self.chain.last().unwrap();
    }

    fn new_transaction(&mut self, sender: String, recipient: String, amount: i8) -> i8 {
        self.current_transactions.push(Transaction {
            sender,
            recipient,
            amount,
        });
        self.last_block().index + 1
    }
}




fn main() {
    let mut blockchian = Blockchain::new();
    blockchian.init();
    let node_identifier = Uuid::new_v4().to_string().replace("-", "");
}