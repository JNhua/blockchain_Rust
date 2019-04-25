extern crate time;
extern crate crypto;
extern crate urlparse;
extern crate requests;
extern crate rustc_serialize;
extern crate uuid;
extern crate canteen;
extern crate serde;
extern crate serde_json;
extern crate argparse;


#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use canteen::{Canteen, Request, Response, Method};
use canteen::utils;

use urlparse::urlparse;
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use ::time::get_time;
use std::rc::Rc;
use std::cell::RefCell;
use requests::*;
use rustc_serialize::json;
use uuid::Uuid;
use canteen::utils::make_response;
use serde_json::json;
use serde::de::Deserializer;
use argparse::{ArgumentParser, Store};

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: i8,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, Serialize, Deserialize)]
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
    unsafe fn init(&mut self) {
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

static mut BC: Blockchain = Blockchain::new();

lazy_static! {
    static ref NI:String = {
        let node_identifier = Uuid::new_v4().to_string().replace("-", "");
        node_identifier
    };
}

fn mine(_: &Request) -> Response {
    let last_block = BC.last_block();
    let last_proof = last_block.proof;
    let proof = Blockchain::proof_of_work(last_proof);

    BC.new_transaction("0".to_string(), NI.to_string(), 1);

    let block = BC.new_block(proof, None);

    #[derive(Debug, Serialize, Deserialize)]
    struct Res {
        #[serde(default)]
        message: String,
        index: i8,
        transactions: Vec<Transaction>,
        proof: i32,
        previous_hash: String,
    }

    let response = Res {
        message: "New Block Forged".to_string(),
        index: block.index,
        transactions: block.transaction,
        proof: block.proof,
        previous_hash: block.previous_hash,
    };

    Response::as_json(&response)
}

fn new_transaction(req: &Request) -> Response {
    let values = req.get_json().unwrap();
    let required = vec!["sender", "recipient", "amount"];
    for k in required {
        if values[k] == json!(null) {
            return make_response("Missing values", "text/plain", 400);
        }
    };

    let index: i8 = BC.new_transaction(values["sender"].to_string(),
                                       values["recipient"].to_string(),
                                       json::decode(&values["amount"].to_string()).unwrap());

    #[derive(Debug, Serialize, Deserialize)]
    struct Res {
        #[serde(default)]
        message: String,
    }

    let response = Res {
        message: "Transacntion will be added to Block ".to_string() + &json::encode(&index).unwrap(),
    };

    Response::as_json(&response)
}

fn full_chain(_: &Request) -> Response {
    #[derive(Debug, Serialize, Deserialize)]
    struct Res {
        #[serde(default)]
        chain: Vec<Block>,
        length: usize,
    }

    let response = Res {
        chain: BC.chain.clone(),
        length: BC.chain.len(),
    };
    Response::as_json(&response)
}

fn register_nodes(req: &Request) -> Response {
    let values = req.get_json().unwrap();
    let nodes = values.get("nodes");
    match nodes {
        None => {
            make_response("Error:Please supply a valid list of nodes.",
                          "text/plain", 400)
        },
        Some(v) => {
            for node in v.as_array().unwrap() {
                BC.register_node(node.as_str().unwrap().to_string());
            }
            #[derive(Debug, Serialize, Deserialize)]
            struct Res {
                #[serde(default)]
                message: String,
                total_nodes: Vec<String>,
            }

            let response = Res {
                message: "New nodes have been added".to_string(),
                total_nodes: BC.nodes,
            };
            Response::as_json(&response)
        }
    }
}

fn consensus(_: &Request) -> Response {
    let replaced = BC.resolve_conflicts();

    match replaced {
        true => {
            #[derive(Debug, Serialize, Deserialize)]
            struct Res {
                #[serde(default)]
                message: String,
                new_chain: Vec<Block>,
            }

            let response = Res {
                message: "Our chain was replaced".to_string(),
                new_chain: BC.chain,
            };
            Response::as_json(&response)
        }
        false => {
            #[derive(Debug, Serialize, Deserialize)]
            struct Res {
                #[serde(default)]
                message: String,
                chain: Vec<Block>,
            }

            let response = Res {
                message: "Our chain is authoritative".to_string(),
                chain: BC.chain,
            };
            Response::as_json(&response)
        }
    }
}


fn main() {
    let mut parser = ArgumentParser::new();
    let mut port = 5000;
    parser.refer(&mut port).add_option(&["-p", "-port"],
                                       Store, "port to listen on");
    parser.parse_args_or_exit();

    let mut cnt = Canteen::new();
    cnt.bind(("127.0.0.1", port));

    cnt.set_default(utils::err_404);
    cnt.add_route("/", &[Method::Get], mine)
        .add_route("/transaction/new", &[Method::Get], new_transaction)
        .add_route("/chain", &[Method::Get], full_chain)
        .add_route("/nodes/register", &[Method::Post], register_nodes)
        .add_route("/nodes/resolve", &[Method::Get], consensus);

    cnt.run();
}