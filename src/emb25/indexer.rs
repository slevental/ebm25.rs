use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::emb25::crypto::{encrypt, encrypt_index_document_key, encrypt_index_document_val, EncryptedDocument, EncryptedDocumentStorage, EncryptedIndexUpdate, EncryptedTerm2Document, SymmetricKey};
use crate::emb25::index::{Term, Term2Document};
use rand::{RngCore, rngs::OsRng};
use crate::{Document, IndexUpdate, tokenize};

struct Keys {
    document_key: SymmetricKey,
    index_key: Vec<u8>,
    value_key: Vec<u8>,
}

impl Keys {
    fn generate_secure_random(size: usize) -> Vec<u8> {
        let mut rng = OsRng::default();
        let mut buffer = vec![0u8; size];
        rng.fill_bytes(&mut buffer);
        buffer
    }

    pub fn new() -> Self {
        Self {
            document_key: SymmetricKey::new(),
            index_key: Self::generate_secure_random(32),
            value_key: Self::generate_secure_random(32),
        }
    }
}

pub struct Indexer {
    keys: Keys,
    dictionary: Dictionary,
    documents: HashMap<u64, Document>,
    index_records: Vec<Term2Document>,
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            keys: Keys::new(),
            dictionary: Dictionary::new(),
            documents: HashMap::new(),
            index_records: Vec::new(),
        }
    }

    pub fn add(&mut self, text: String) -> Document {
        // generate a random key for the document
        let mut rng = OsRng::default();
        let id = rng.next_u64();

        // store document
        let document = Document {
            id,
            title: "".to_string(),
            content: text.clone(),
        };

        // get terms from text
        let tokens = tokenize(&text);

        for token in tokens {
            let term = Term::new(token);
            let freq = self.dictionary.add_or_get(term.clone());

            self.index_records.push(Term2Document {
                term,
                freq,
                document: document.clone(),
            });
        }

        document
    }

    pub fn get_encrypted_doc_storage(&self) -> EncryptedDocumentStorage {
        let mut encrypted_docs = EncryptedDocumentStorage::new();

        for document in self.documents.values() {
            let enc_doc = encrypt(document, &self.keys.document_key);
            encrypted_docs.add(document.id, enc_doc);
        }

        encrypted_docs
    }

    pub fn get_encrypted_index(&self) -> EncryptedIndexUpdate {
        EncryptedIndexUpdate::insert(
            self.index_records.iter().map(|record| {
                let term = record.term.clone();
                let freq = record.freq;
                let document = record.document.clone();
                let key = encrypt_index_document_key(&term, freq, &self.keys.index_key);
                let value = encrypt_index_document_val(&term, freq, document.id, &self.keys.value_key);

                EncryptedTerm2Document::new(key, value)
            }).collect())
    }
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

    pub fn get(&self, term: &Term) -> Option<&u32> {
        self.terms.get(term)
    }
}


#[cfg(test)]
mod tests {
    use crate::emb25::crypto::{EncryptedIndex, get_document_id};
    use super::*;

    #[test]
    fn test_add() {
        let mut indexer = Indexer::new();
        let text = "This is a test".to_string();
        let document = indexer.add(text);
        assert_eq!(document.content, "This is a test");
    }

    #[test]
    fn test_get_encrypted_doc_storage() {
        let mut indexer = Indexer::new();
        let mut index = EncryptedIndex::new();
        let text = "This is a test".to_string();
        let document = indexer.add(text);

        let encrypted_doc_storage = indexer.get_encrypted_doc_storage();
        index.update(&indexer.get_encrypted_index());

        // search
        let term = Term::new("This".to_string());
        let freq = indexer.dictionary.get(&term).unwrap();
        let key_req = encrypt_index_document_key(&term, *freq, &indexer.keys.index_key);

        let val_res = index.get(&key_req).unwrap();
        let id = get_document_id(&term, *freq, val_res.clone(), &indexer.keys.value_key);

        assert_eq!(id, document.id);
    }
}