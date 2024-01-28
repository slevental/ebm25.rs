use ebm25::{EncryptedDocument, Indexer, Query};
use std::collections::HashMap;
use std::ops::Add;

#[tokio::main]
async fn main() {
    let mut indexer = Indexer::new();

    let documents: Vec<String> = vec![
        "The quick brown fox jumps over the lazy dog".to_string(),
        "The quick brown fox jumps over the quick dog".to_string(),
        "Brown fox brown dog".to_string(),
        "Brown fox lazy dog".to_string(),
        "Lazy dog quick brown fox".to_string(),
        "Brown dog lazy fox".to_string(),
        "The quick brown fox and the quick blue hare".to_string(),
    ];

    for document in documents {
        indexer.add(document);
    }

    let index_update = indexer.get_encrypted_index();
    let storage = indexer.get_encrypted_doc_storage();
    let client = reqwest::Client::new();

    for document in storage.documents.values() {
        let url = String::from("http://127.0.0.1:8080/index/").add(&document.id.to_string());
        client.post(url).json(&document).send().await.unwrap();
    }

    // now update the index
    client
        .post("http://127.0.0.1:8080/index")
        .json(&index_update)
        .send()
        .await
        .unwrap();

    // Searching:

    // Create a query
    let mut query = indexer.query("quick brown".to_string());

    // Send encrypted keys to server
    let response = client
        .post("http://127.0.0.1:8080/search")
        .json(&query.query)
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        let response: Vec<Vec<u8>> = response.json().await.unwrap();
        let mut doc_id_to_score = HashMap::new();
        let bm25 = indexer.bm25();
        for i in 0..response.len() {
            let val = &response[i];
            let mut term = &mut query.terms[i];
            let meta = indexer.meta(term, val);
            let doc_freq = indexer.dictionary.freq(&term.term).unwrap();
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
        for doc_id in doc_ids {
            let score = doc_id_to_score.get(&doc_id).unwrap();
            let url = String::from("http://127.0.0.1:8080/index/").add(&doc_id.to_string());
            // request the documents from the server
            let response = client.get(url).send().await.unwrap();

            if response.status().is_success() {
                let response: EncryptedDocument = response.json().await.unwrap();
                let d = indexer.decrypt(&response);
                println!(
                    "doc_id={} score={} title={} content={}",
                    doc_id, score, d.title, d.content
                );
            }
        }
    } else {
        println!("Error: {:?}", response.status());
        // Handle error response
    }
}
