use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::emb25::index::Term;

struct SecretKey {

}

pub struct Indexer {

}


#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Dictionary {
    // terms with frequencies
    pub terms: HashMap<Term, u32>,
}


impl Dictionary {
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
        }
    }

    pub fn add_or_get(&mut self, term: Term) -> u32 {
        let entry = self.terms.entry(term).or_insert(0);
        *entry += 1;
        *entry
    }
}
