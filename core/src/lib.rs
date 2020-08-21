pub mod block;
pub mod blockchain;
pub mod pow;
pub mod bcdb;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
