use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ebm25::{
    Document, EncryptedDocument, EncryptedDocumentStorage, EncryptedIndex, EncryptedIndexUpdate,
    Query,
};
use std::collections::HashMap;
use std::sync::Mutex;

// This is an encrypted server state
// it contains search index and document storage

struct ServerState {
    storage: Mutex<EncryptedDocumentStorage>,
    index: Mutex<EncryptedIndex>,
}

// Handler to index a document

async fn upload_document(
    doc: web::Json<EncryptedDocument>,
    data: web::Data<ServerState>,
) -> impl Responder {
    let mut db = data.storage.lock().unwrap();
    let document = doc.into_inner();
    let id = document.id;
    db.add(document);
    println!(
        "Document id={} was indexed (Total={:?})",
        id,
        db.documents.len()
    );
    HttpResponse::Ok().body("Document indexed")
}

async fn get_document(request: web::Path<u64>, data: web::Data<ServerState>) -> impl Responder {
    let db = data.storage.lock().unwrap();
    let id = request.into_inner();
    let doc: Option<EncryptedDocument> = db.get(id).map(|x| x.clone());
    if doc.is_none() {
        return HttpResponse::NotFound().body("Document not found");
    }
    HttpResponse::Ok().json(doc)
}

async fn update_index(
    upd: web::Json<EncryptedIndexUpdate>,
    data: web::Data<ServerState>,
) -> impl Responder {
    let mut index = data.index.lock().unwrap();
    let update = &upd.into_inner();
    index.update(update);
    println!(
        "Index was updated with {} records (Total={})",
        update.len(),
        index.len()
    );
    HttpResponse::Ok().body("Index updated")
}

// Handler to search for a document
async fn search_doc(
    request: web::Json<Vec<Vec<u8>>>,
    data: web::Data<ServerState>,
) -> impl Responder {
    let index = data.index.lock().unwrap();
    let query = request.into_inner();
    let mut encoded_data = Vec::new();
    let mut found = 0;
    for i in 0..query.len() {
        let term = &query[i];
        if let Some(segment) = index.get(term) {
            encoded_data.push(segment.clone());
            found += 1;
            continue;
        } else {
            println!("Term not found: {:?}", term);
            encoded_data.push(vec![]);
        }
    }
    println!("Found {} out of {} terms", found, query.len());
    HttpResponse::Ok().json(encoded_data)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let shared_data = web::Data::new(ServerState {
        index: Mutex::new(EncryptedIndex::new()),
        storage: Mutex::new(EncryptedDocumentStorage::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .route("/index/{id}", web::post().to(upload_document))
            .route("/index/{id}", web::get().to(get_document))
            .route("/index", web::post().to(update_index))
            .route("/search", web::post().to(search_doc))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
