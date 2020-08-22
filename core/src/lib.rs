pub mod block;
pub mod blockchain;
pub mod pow;
pub mod bcdb;
pub mod transaction;
pub mod account;
pub mod miner;
pub mod mycore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
