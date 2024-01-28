use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ebm25::{
    Document, EncryptedDocument, EncryptedDocumentStorage, EncryptedIndex, EncryptedIndexUpdate,
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
    db.add(doc.into_inner());
    HttpResponse::Ok().body("Document indexed")
}

async fn get_document(request: web::Path<u64>, data: web::Data<ServerState>) -> impl Responder {
    let db = data.storage.lock().unwrap();
    let id = request.into_inner();
    let doc: Option<EncryptedDocument> = db.get(id).map(|x| x.clone());
    HttpResponse::Ok().json(doc)
}

async fn update_index(
    upd: web::Json<EncryptedIndexUpdate>,
    data: web::Data<ServerState>,
) -> impl Responder {
    let mut index = data.index.lock().unwrap();
    index.update(&upd.into_inner());
    HttpResponse::Ok().body("Index updated")
}

// Handler to search for a document
async fn search_doc(
    request: web::Path<Vec<Vec<u8>>>,
    data: web::Data<ServerState>,
) -> impl Responder {
    let index = data.index.lock().unwrap();
    let id = request.into_inner();
    let mut encoded_data = Vec::new();
    for i in 0..id.len() {
        let segment = index.get(&id[i]).map(|x| x.clone());
        if let Some(ref s) = segment {
            encoded_data.extend_from_slice(&s);
        }
        let length: [u8; 4] = segment.map(|x| x.len() as u32).unwrap_or(0).to_be_bytes();
        encoded_data.extend_from_slice(&length);
    }
    HttpResponse::Ok().body(encoded_data)
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
            .route("/search", web::get().to(search_doc))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
