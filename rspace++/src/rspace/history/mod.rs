pub mod history;
pub mod history_reader;
pub mod history_repository;
pub mod history_action;
pub mod instances;
pub mod radix_tree;
pub mod roots_store;
pub mod root_repository;
pub mod cold_store;
pub mod history_repository_impl;

// PartialEq needed here for testing purposes
#[derive(Debug, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
