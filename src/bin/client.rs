use ebm25::{Indexer, Query};
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
    let query = indexer.query("quick brown".to_string());

    // Send encrypted keys to server
    let response = client
        .post("http://127.0.0.1:8080/search")
        .json(&query.query)
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        let response: Vec<Vec<u8>> = response.json().await.unwrap();

        for i in 0..response.len() {
            let val = &response[i];
            let term = &query.terms[i];
            let meta = indexer.meta(term, val);
            println!("Found document: {:?}", meta)
        }
    } else {
        println!("Error: {:?}", response.status());
        // Handle error response
    }
}
