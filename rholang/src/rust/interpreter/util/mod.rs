use models::rhoapi::{Bundle, Expr, Match, New, Receive, Send};

pub mod address_tools;
pub mod base58;
pub mod rev_address;

// Helper enum. This is 'GeneratedMessage' in Scala
#[derive(Clone)]
pub enum GeneratedMessage {
    Send(Send),
    Receive(Receive),
    New(New),
    Match(Match),
    Bundle(Bundle),
    Expr(Expr),
}
