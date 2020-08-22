pub mod block;
pub mod blockchain;
pub mod pow;
pub mod bcdb;
pub mod transaction;
pub mod account;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
