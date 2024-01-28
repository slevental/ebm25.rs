use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct Document {
    pub id: u64,
    pub title: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Term {
    pub term: String,
}

impl Term {
    pub fn new(term: String) -> Self {
        Self { term }
    }
}

#[derive(PartialEq, Debug)]
pub struct Term2Document {
    pub term: Term,
    pub freq: u32,
    pub document: Document,
}

#[derive(PartialEq, Debug)]
pub struct IndexUpdate {
    pub relations: Vec<Term2Document>,
}
