use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng, Nonce}, Aes256Gcm, Key, AesGcm};
use aes_gcm::aead::consts::U12;
use aes_gcm::aes::Aes256;
use crate::Document;

struct SymmetricKey {
    key: Key<Aes256Gcm>,
}

#[derive(Debug, PartialEq)]
struct EncryptedDocument {
    nonce: Nonce<AesGcm<Aes256, U12>>,
    ciphertext: Vec<u8>,
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
        nonce,
        ciphertext,
    }
}

pub fn decrypt(document: &EncryptedDocument, key: &SymmetricKey) -> Document {
    // decrypt the ciphertext using the nonce and key
    // deserialize the byte array using serde
    // return the document

    let cipher = Aes256Gcm::new(&key.key);
    let plaintext = cipher.decrypt(&document.nonce, document.ciphertext.as_ref()).unwrap();
    let document: Document = serde_json::from_slice(&plaintext).unwrap();

    document
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = SymmetricKey {
            key: Aes256Gcm::generate_key(OsRng)
        };
        let document = Document {
            id: "some id".to_string(),
            title: "title".to_string(),
            content: "body".to_string(),
        };
        let encrypted_document = encrypt(&document, &key);
        let decrypted_document = decrypt(&encrypted_document, &key);

        assert_eq!(document, decrypted_document);
    }
}