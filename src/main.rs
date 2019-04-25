extern crate time;
extern crate crypto;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate canteen;
extern crate urlparse;
extern crate requests;
extern crate rustc_serialize;
extern crate uuid;

use urlparse::urlparse;
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use ::time::*;
use std::rc::Rc;
use requests::*;
use rustc_serialize::json;
use uuid::*;


struct Transaction {
    sender: String,
    recipient: String,
    amount: i8,
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
    /*
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
    */

    fn myhash(block: Block) -> String {
        // create a SHA3-256 object
        let mut hasher = Sha3::sha3_256();
        // write input message
        hasher.input_str(&json::encode(&block).unwrap());
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

    fn resolve_conflicts(&mut self) -> bool {
        let neighbours = &self.nodes;
        let mut new_chain = vec![];
        let mut max_length = self.chain.len();

        for node in neighbours {
            let response =
                requests::get("http://" + String::from(node) + "/chain").unwrap();

            if response.status_code() == 200 {
                let res_json = response.json().unwrap();
                let length = res_json["length"].as_usize().unwrap();
                let chain = res_json["chain"];

                if length > max_length && valid_chain(chain) {
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

    fn new_transaction(&mut self, sender: String, recipient: String, amount: i8) -> i8 {
        self.current_transactions.push(Transaction {
            sender,
            recipient,
            amount,
        });
        self.last_block().index + 1
    }

    fn last_block(&self) -> &Block {
        return self.chain.last().unwrap();
    }

    fn timestamp() -> i64 {
        let timespec = time::get_time();
        timespec.sec * 1000 + (timespec.nsec as f64 / 1000.0 / 1000.0) as i64
    }

    fn proof_of_work(last_proof: i32) -> i32 {
        let proof = 0;
        while valid_proof(last_proof, proof) == false {
            proof += 1;
        }
        proof
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

use canteen::*;
use canteen::utils;
use canteen::{Response, Request};

fn mine(req: &Request) -> Response {
    let mut res = Response::new();

    res.set_status(200);
    res.set_content_type("text/plain");
    res.append("Hello, world!");

    res
}


fn main() {
    let mut blockchian = Blockchain::new();
    blockchian.init();
    let mut cnt = Canteen::new();
    let node_identifier = String::from(uuid::Uuid::new_v4()).replace("-", "");

    // bind to the listening address
    cnt.bind(("127.0.0.1", 5000));

    // set the default route handler to show a 404 message
    cnt.set_default(utils::err_404);

    // respond to requests to / with "Hello, world!"
    cnt.add_route("/mine", &[Method::Get], mine);

    cnt.run();
}