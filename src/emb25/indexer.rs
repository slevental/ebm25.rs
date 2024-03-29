use crate::emb25::crypto::{
    decrypt, encrypt, encrypt_index_key, encrypt_index_value, get_document_meta, DocumentMeta,
    EncryptedDocument, EncryptedDocumentStorage, EncryptedIndexUpdate, EncryptedTerm2Document,
    SymmetricKey,
};
use crate::emb25::index::{Term, Term2Document};
use crate::{group_by, tokenize, Document, IndexUpdate};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Query {
    pub terms: Vec<Term>,
    pub query: Vec<Vec<u8>>,
}

pub struct Indexer {
    // client only needs to persist dictionary
    // and total document size, the rest can be
    // reconstructed from the server state
    pub dictionary: Dictionary,

    keys: Keys,
    documents: HashMap<u64, Document>,
    index_records: Vec<Term2Document>,

    total_document_size: u64,
}

// replace with enum later
pub struct BM25 {
    k1: f64,
    b: f64,
    avgdl: f64,
    doc_count: u64,
}

impl BM25 {
    fn new(k1: f64, b: f64, avgdl: f64, doc_count: u64) -> BM25 {
        BM25 {
            k1,
            b,
            avgdl,
            doc_count,
        }
    }

    fn idf(&self, doc_count: u64, doc_freq: u64) -> f64 {
        (1. + (doc_count as f64 - doc_freq as f64 + 0.5) / (doc_freq as f64 + 0.5)).ln()
    }

    pub fn score(&self, doc_len: u64, term_freq: u64, doc_freq: u64) -> f64 {
        let idf = self.idf(self.doc_count, doc_freq);
        let term_freq = term_freq as f64;
        let doc_len = doc_len as f64;

        idf * (term_freq * (self.k1 + 1.0))
            / (term_freq + self.k1 * (1.0 - self.b + self.b * doc_len / self.avgdl))
    }
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            dictionary: Dictionary::new(),
            keys: Keys::new(),
            documents: HashMap::new(),
            index_records: Vec::new(),
            total_document_size: 0u64,
        }
    }

    pub fn meta(&self, term: &Term, value: &Vec<u8>) -> DocumentMeta {
        get_document_meta(&term, value, &self.keys.value_key)
    }

    pub fn decrypt(&self, enc_doc: &EncryptedDocument) -> Document {
        let decr = decrypt(enc_doc, &self.keys.document_key);
        decr
    }

    pub fn bm25(&self) -> BM25 {
        let avgdl = self.total_document_size as f64 / self.documents.len() as f64;
        BM25::new(1.2, 0.75, avgdl, self.documents.len() as u64)
    }

    pub fn query(&self, text: String) -> Query {
        let tokens = tokenize(&text);
        let grouped = group_by(&tokens);
        let mut terms = Vec::new();
        let mut query = Vec::new();

        for (token, count) in grouped.iter() {
            let documents = self.dictionary.freq(token).unwrap_or(&0);

            // to make server document storage safer we cannot let the server
            // enumerate through the list of documents in index, but we can
            // optimize it eventually by sending hash function state that could be
            // updated with indices until documents remain in the index
            for i in 0..*documents {
                let id = i + 1;
                let mut term = Term::new(token.clone(), id);
                term.score_mult(*count as f64);
                query.push(encrypt_index_key(&term, &self.keys.index_key));
                terms.push(term);
            }
        }

        Query { terms, query }
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

        self.documents.insert(id, document.clone());
        self.total_document_size += text.len() as u64;

        // get terms from text
        let tokens = tokenize(&text);

        // group by term and count the frequency
        let token_freq = group_by(&tokens);

        for (token, f) in token_freq.iter() {
            let id = self.dictionary.add_or_get(token.clone());
            let term = Term::new(token.clone(), id);

            self.index_records.push(Term2Document {
                term,
                freq: *f,
                document: document.clone(),
            });
        }

        document
    }

    pub fn get_encrypted_doc_storage(&self) -> EncryptedDocumentStorage {
        let mut encrypted_docs = EncryptedDocumentStorage::new();

        for document in self.documents.values() {
            let enc_doc = encrypt(document, &self.keys.document_key);
            encrypted_docs.add(enc_doc);
        }

        encrypted_docs
    }

    pub fn get_encrypted_index(&self) -> EncryptedIndexUpdate {
        EncryptedIndexUpdate::insert(
            self.index_records
                .iter()
                .map(|record| {
                    let term = record.term.clone();
                    let freq = record.freq;
                    let document = record.document.clone();
                    let key = encrypt_index_key(&term, &self.keys.index_key);
                    let meta = DocumentMeta::new(document.id, document.content.len() as u64, freq);
                    let value = encrypt_index_value(&term, &meta, &self.keys.value_key);

                    EncryptedTerm2Document::new(key, value)
                })
                .collect(),
        )
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Dictionary {
    // terms with frequencies
    pub terms: HashMap<String, u64>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
        }
    }

    pub fn add_or_get(&mut self, term: String) -> u64 {
        let entry = self.terms.entry(term).or_insert(0);
        *entry += 1;
        *entry
    }

    pub fn freq(&self, term: &str) -> Option<&u64> {
        self.terms.get(term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emb25::crypto::{decrypt, get_document_meta, EncryptedIndex};

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
        let term = Term::new("This".to_string(), 1);

        let key_req = encrypt_index_key(&term, &indexer.keys.index_key);
        let val_res = index.get(&key_req).unwrap();
        let meta = get_document_meta(&term, &val_res, &indexer.keys.value_key);

        assert_eq!(meta.id, document.id);

        let decr = decrypt(
            encrypted_doc_storage.get(meta.id).unwrap(),
            &indexer.keys.document_key,
        );
        assert_eq!(decr.content, document.content)
    }
}
