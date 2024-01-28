use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Document {
    pub id: u64,
    pub title: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Debug)]
pub struct Term {
    pub term: String,
}

#[derive(PartialEq, Debug)]
pub struct Term2Document {
    pub term: Term,
    pub freq: u32,
    pub document: Document,
}

#[derive(PartialEq, Debug)]
pub struct IndexUpdate {
    pub relations: Vec<Term2Document>
}
