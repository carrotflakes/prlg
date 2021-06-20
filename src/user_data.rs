#[derive(Debug)]
pub enum UserData {
    Variable(String),
    Wildcard,
    Symbol(String),
    Term(Vec<UserData>),
}
