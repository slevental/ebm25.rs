use ebm25::{BM25, Document, EncryptedDocument, Term, DocumentMeta, Indexer, Query};
use std::collections::HashMap;
use std::ops::Add;



struct Client {
    indexer: Indexer,
    url: String,
}

impl Client {
    pub fn new(url: String) -> Self {
        Self {
            indexer: Indexer::new(),
            url,
        }
    }

    pub fn add(&mut self, document: String) {
        self.indexer.add(document);
    }

    pub fn query(&self, query: String) -> Query {
        self.indexer.query(query)
    }

    pub fn bm25(&self) -> BM25 {
        self.indexer.bm25()
    }

    pub fn meta(&self, term: &Term, val: &Vec<u8>) -> DocumentMeta {
        self.indexer.meta(term, val)
    }

    pub fn decrypt(&self, document: &EncryptedDocument) -> Document {
        self.indexer.decrypt(document)
    }

    pub async fn flush(&self) {
        let index_update = self.indexer.get_encrypted_index();
        let storage = self.indexer.get_encrypted_doc_storage();
        let client = reqwest::Client::new();

        for document in storage.documents.values() {
            let url = self.url.clone().add("/index/").add(&document.id.to_string());
            client.post(url).json(&document).send().await.unwrap();
        }

        // now update the index
        client
            .post(self.url.clone().add("/index"))
            .json(&index_update)
            .send()
            .await
            .unwrap();
    }

    pub async fn search(&self, query: &mut Query, top_k: u32) -> Vec<Document> {
        let client = reqwest::Client::new();

        // Send encrypted keys to server
        let response = client
            .post(self.url.clone().add("/search"))
            .json(&query.query)
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let response: Vec<Vec<u8>> = response.json().await.unwrap();
            let mut doc_id_to_score = HashMap::new();
            let bm25 = self.indexer.bm25();
            for i in 0..response.len() {
                let val = &response[i];
                let mut term = &mut query.terms[i];
                let meta = self.indexer.meta(term, val);
                let doc_freq = self.indexer.dictionary.freq(&term.term).unwrap();
                let score = bm25.score(meta.size, meta.f, *doc_freq);
                println!("term={} score={}", term.term, score);
                term.score_mult(score);

                // add to score for this doc id
                *doc_id_to_score.entry(meta.id.clone()).or_insert(0.) += score;
            }

            // sort by score
            let mut doc_ids: Vec<u64> = doc_id_to_score.keys().cloned().collect();
            doc_ids.sort_by(|a, b| {
                let score_a = doc_id_to_score.get(a).unwrap();
                let score_b = doc_id_to_score.get(b).unwrap();
                score_b.partial_cmp(score_a).unwrap()
            });

            // print results
            let mut results = Vec::new();

            // only return top k results (if there are that many)
            let top_k = if doc_ids.len() < top_k as usize {
                doc_ids.len()
            } else {
                top_k as usize
            };
            let doc_ids = doc_ids[0..top_k as usize].to_vec();

            for doc_id in doc_ids {
                let url = self.url.clone().add("/index/").add(&doc_id.to_string());
                // request the documents from the server
                let response = client.get(url).send().await.unwrap();

                if response.status().is_success() {
                    let response: EncryptedDocument = response.json().await.unwrap();
                    let d = self.indexer.decrypt(&response);
                    results.push(d);
                }
            }

            results
        } else {
            println!("Error: {:?}", response.status());
            // Handle error response
            Vec::new()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new("http://localhost:8080".to_string());

    let documents: Vec<String> = vec![
        "The quick brown fox jumps over the lazy dog".to_string(),
        "The quick brown fox jumps over the quick dog".to_string(),
        "Brown fox brown dog".to_string(),
        "Magic the gathering".to_string(),
        "Brown fox lazy dog".to_string(),
        "Lazy dog quick brown fox".to_string(),
        "Brown dog lazy fox".to_string(),
        "The quick brown fox and the quick blue hare".to_string(),
    ];

    for document in documents {
        client.add(document);
    }

    client.flush().await;

    let mut query = client.query("gathering".to_string());
    let result = client.search(&mut query, 5).await;

    println!("Result: {:?}", result);

    Ok(())
}
