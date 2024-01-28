use std::collections::HashMap;
use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng, Nonce}, Aes256Gcm, Key, AesGcm};
use aes_gcm::aead::consts::U12;
use aes_gcm::aes::Aes256;
use sha3::{Digest, Sha3_256, Sha3_256Core};
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use sha3::digest::{DynDigest, Update};
use sha3::digest::core_api::CoreWrapper;
use crate::Document;
use crate::emb25::index::{IndexUpdate, Term, Term2Document};

#[derive(Clone)]
pub struct SymmetricKey {
    key: Key<Aes256Gcm>,
}

impl SymmetricKey {
    pub fn new() -> Self {
        Self {
            key: Aes256Gcm::generate_key(OsRng),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EncryptedDocument {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EncryptedIndex {
    index: HashMap<Vec<u8>, Vec<u8>>,
}

impl EncryptedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn update(&mut self, index_update: &EncryptedIndexUpdate) {
        index_update.add.iter().for_each(|r| {
            self.index.insert(r.t.clone(), r.d.clone());
        });
    }

    pub fn add(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.index.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.index.get(key)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EncryptedDocumentStorage {
    documents: HashMap<u64, EncryptedDocument>,
}

impl EncryptedDocumentStorage {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: u64, document: EncryptedDocument) {
        self.documents.insert(id, document);
    }

    pub fn get(&self, id: u64) -> Option<&EncryptedDocument> {
        self.documents.get(&id)
    }
}

#[derive(PartialEq, Debug)]
pub struct EncryptedTerm2Document {
    t: Vec<u8>,
    d: Vec<u8>,
}

impl EncryptedTerm2Document {
    pub fn new(term: Vec<u8>, document: Vec<u8>) -> Self {
        Self {
            t: term,
            d: document,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct EncryptedIndexUpdate {
    add: Vec<EncryptedTerm2Document>,
}

impl EncryptedIndexUpdate {
    pub fn insert(vec: Vec<EncryptedTerm2Document>) -> Self {
        Self { add: vec }
    }
}

impl EncryptedIndexUpdate {
    pub fn new() -> Self {
        Self {
            add: Vec::new(),
        }
    }

    pub fn add(&mut self, term: Vec<u8>, document: Vec<u8>) {
        self.add.push(EncryptedTerm2Document {
            t: term,
            d: document,
        });
    }
}

fn initialize_hasher_sha256(term: &&Term, freq: &u32, key: &[u8]) -> CoreWrapper<Sha3_256Core> {
    let mut hasher = Sha3_256::new();
    Digest::update(&mut hasher, key);
    Digest::update(&mut hasher, &term.term.as_bytes());
    Digest::update(&mut hasher, &freq.to_be_bytes());
    hasher
}

pub fn encrypt_index_document_key(term: &Term, freq: u32, key: &[u8]) -> Vec<u8> {
    let hasher = initialize_hasher_sha256(&term, &freq, key);
    hasher.finalize().to_vec()
}

pub fn encrypt_index_document_val(term: &Term, freq: u32, doc_id: u64, key: &[u8]) -> Vec<u8> {
    let mut hasher = initialize_hasher_sha256(&term, &freq, key);

    let value = hasher.finalize();
    let value = &value[0..8];
    let value = u64::from_be_bytes(value.try_into().unwrap());
    (value ^ doc_id).to_be_bytes().to_vec()
}

pub fn get_document_id(term: &Term, freq: u32, value: Vec<u8>, key: &[u8]) -> u64 {
    let hasher = initialize_hasher_sha256(&term, &freq, key);

    let h = hasher.finalize();
    let h = &h[0..8];
    let h = u64::from_be_bytes(h.try_into().unwrap());

    if value.len() != 8 {
        panic!("value length is not 8");
    }

    let array: [u8; 8] = value.try_into().expect("Exact length checked");
    let v = u64::from_be_bytes(array);
    h ^ v
}

pub fn encrypt_index_update(index_update: &IndexUpdate, k1: &[u8], k2: &[u8]) -> EncryptedIndexUpdate {
    let mut encr = EncryptedIndexUpdate::new();

    index_update.relations.iter().for_each(|r| {
        let key_vec = encrypt_index_document_key(&r.term, r.freq, k1);
        let value_vec = encrypt_index_document_val(&r.term, r.freq, r.document.id, k2);
        encr.add(key_vec, value_vec);
    });

    encr
}

pub fn encrypt(document: &Document, key: &SymmetricKey) -> EncryptedDocument {
    // serialize document using serde to a byte array
    // encrypt the byte array
    // return the nonce and ciphertext

    let cipher = Aes256Gcm::new(&key.key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let bytes = serde_json::to_vec(&document).unwrap();
    let ciphertext = cipher.encrypt(&nonce, bytes.as_ref()).unwrap();

    EncryptedDocument {
        nonce: nonce.to_vec(),
        ciphertext,
    }
}

pub fn decrypt(document: &EncryptedDocument, key: &SymmetricKey) -> Document {
    // decrypt the ciphertext using the nonce and key
    // deserialize the byte array using serde
    // return the document

    let cipher = Aes256Gcm::new(&key.key);
    let nonce = Nonce::<AesGcm<Aes256, U12>>::from_slice(&document.nonce);
    let plaintext = cipher.decrypt(nonce, document.ciphertext.as_ref()).unwrap();
    let document: Document = serde_json::from_slice(&plaintext).unwrap();

    document
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing_and_xor() {
        let t = &Term { term: "term".to_string() };
        let key = hex!("1234567890");
        let doc_id = 78361473624;
        let hash = encrypt_index_document_val(t, 42, doc_id, &key);
        let id = get_document_id(t, 42, hash, &key);

        assert_eq!(id, doc_id);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = SymmetricKey::new();

        let document = Document {
            id: 42,
            title: "title".to_string(),
            content: "body".to_string(),
        };
        let encrypted_document = encrypt(&document, &key);
        let decrypted_document = decrypt(&encrypted_document, &key);

        assert_eq!(document, decrypted_document);
    }
}