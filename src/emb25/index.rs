use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct Document {
    pub id: u64,
    pub title: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct Term {
    pub term: String,

    // Term ID is a special parameter that mean the
    // Seq number of the document this term is seen in
    // this is done to avoid collisions in the index and
    // prevent it from statistical attacks
    pub id: u64,

    pub score: f64,
}

impl Term {
    pub fn new(term: String, id: u64) -> Self {
        Self {
            term,
            id,
            score: 1.0,
        }
    }

    pub fn score_mult(&mut self, score: f64) {
        self.score *= score;
    }
}

#[derive(PartialEq, Debug)]
pub struct Term2Document {
    pub term: Term,

    // How many times this term is seen in the document
    pub freq: u64,

    pub document: Document,
}

#[derive(PartialEq, Debug)]
pub struct IndexUpdate {
    pub relations: Vec<Term2Document>,
}
