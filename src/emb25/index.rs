use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
}

pub struct Index {
    index: Mutex<HashMap<String, String>>,
    db: Mutex<HashMap<String, Document>>,
}

pub struct Term {

}
pub struct Query {

}