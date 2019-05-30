pub trait DatabaseTrait {
    fn open(dbname: &str) -> Self;
    fn close(&self);
}
