use crate::emb25::index::{IndexUpdate, Term, Term2Document};
use crate::Document;
use aes_gcm::aead::consts::U12;
use aes_gcm::aes::Aes256;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, Nonce, OsRng},
    Aes256Gcm, AesGcm, Key,
};
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use sha3::digest::core_api::CoreWrapper;
use sha3::digest::{DynDigest, Update};
use sha3::{Digest, Sha3_256, Sha3_256Core};
use std::collections::HashMap;

#[derive(Clone)]
pub struct SymmetricKey {
    key: Key<Aes256Gcm>,
}

// Struct to store the metadata of a document
// id: is the id of the document in the database
// size: is the size of the document in bytes
// f: is the frequency of term that was used in the search in this document (see. TF of tf-idf)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DocumentMeta {
    pub id: u64,
    // id of the document in the
    pub size: u64,
    pub f: u64,
}

impl DocumentMeta {
    pub fn new(id: u64, size: u64, f: u64) -> Self {
        Self { id, size, f }
    }
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
        Self { add: Vec::new() }
    }

    pub fn add(&mut self, term: Vec<u8>, document: Vec<u8>) {
        self.add.push(EncryptedTerm2Document {
            t: term,
            d: document,
        });
    }
}

fn initialize_sha256(term: &Term, key: &[u8]) -> CoreWrapper<Sha3_256Core> {
    let mut hasher = Sha3_256::new();
    Digest::update(&mut hasher, key);
    Digest::update(&mut hasher, &term.term.as_bytes());
    hasher
}

fn initialize_hasher_sha256(term: &Term, key: &[u8]) -> CoreWrapper<Sha3_256Core> {
    let mut hasher = initialize_sha256(term, key);
    Digest::update(&mut hasher, &term.id.to_be_bytes());
    hasher
}

pub fn encrypt_index_key(term: &Term, key: &[u8]) -> Vec<u8> {
    let hasher = initialize_hasher_sha256(&term, key);
    hasher.finalize().to_vec()
}

pub fn encrypt_index_value(term: &Term, meta: &DocumentMeta, key: &[u8]) -> Vec<u8> {
    let mut hasher = initialize_hasher_sha256(term, key);

    let arr = hasher.finalize();
    let p1 = u64::from_be_bytes(arr[0..8].try_into().unwrap());
    let p2 = u64::from_be_bytes(arr[8..16].try_into().unwrap());
    let p3 = u64::from_be_bytes(arr[16..24].try_into().unwrap());

    let mut v = Vec::new();
    v.extend_from_slice(&(p1 ^ meta.id).to_be_bytes());
    v.extend_from_slice(&(p2 ^ meta.f).to_be_bytes());
    v.extend_from_slice(&(p3 ^ meta.size).to_be_bytes());
    v
}

pub fn get_document_meta(term: &Term, value: Vec<u8>, key: &[u8]) -> DocumentMeta {
    let hasher = initialize_hasher_sha256(&term, key);

    let h = hasher.finalize();
    let id_xor = u64::from_be_bytes(h[0..8].try_into().unwrap());
    let fr_xor = u64::from_be_bytes(h[8..16].try_into().unwrap());
    let si_xor = u64::from_be_bytes(h[16..24].try_into().unwrap());

    if value.len() != 24 {
        panic!("value length is not 8");
    }

    let mut p1: [u8; 8] = [0; 8];
    let mut p2: [u8; 8] = [0; 8];
    let mut p3: [u8; 8] = [0; 8];

    p1.copy_from_slice(&value[0..8]);
    p2.copy_from_slice(&value[8..16]);
    p3.copy_from_slice(&value[16..24]);

    let p1 = u64::from_be_bytes(p1);
    let p2 = u64::from_be_bytes(p2);
    let p3 = u64::from_be_bytes(p3);

    DocumentMeta {
        id: id_xor ^ p1,
        f: fr_xor ^ p2,
        size: si_xor ^ p3,
    }
}

pub fn encrypt_index_update(
    index_update: &IndexUpdate,
    k1: &[u8],
    k2: &[u8],
) -> EncryptedIndexUpdate {
    let mut encr = EncryptedIndexUpdate::new();

    index_update.relations.iter().for_each(|r| {
        let key_vec = encrypt_index_key(&r.term, k1);
        let meta = DocumentMeta::new(
            r.document.id,
            r.document.content.len() as u64,
            r.freq as u64,
        );
        let value_vec = encrypt_index_value(&r.term, &meta, k2);
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
        let t = &Term {
            term: "term".to_string(),
            id: 42,
        };
        let key = hex!("1234567890");

        let meta = DocumentMeta::new(78361473624, 523232, 42484759348);

        let hash = encrypt_index_value(t, &meta, &key);
        let meta2 = get_document_meta(t, hash, &key);

        assert_eq!(meta, meta2);
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
