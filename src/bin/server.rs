use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ebm25::Document;
use std::collections::HashMap;
use std::sync::Mutex;

// Shared state for indexing documents
struct IndexState {
    index: Mutex<HashMap<String, String>>,
}

// Handler to index a document
async fn index_doc(doc: web::Json<Document>, data: web::Data<IndexState>) -> impl Responder {
    let mut index = data.index.lock().unwrap();
    index.insert(doc.id.clone(), doc.content.clone());
    HttpResponse::Ok().body("Document indexed")
}

// Handler to search for a document
async fn search_doc(id: web::Path<String>, data: web::Data<IndexState>) -> impl Responder {
    let index = data.index.lock().unwrap();
    let id = id.into_inner();
    if let Some(content) = index.get(&id) {
        HttpResponse::Ok().body(content.clone())
    } else {
        HttpResponse::NotFound().body("Document not found")
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let shared_data = web::Data::new(IndexState {
        index: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(shared_data.clone())
            .route("/index", web::post().to(index_doc))
            .route("/search/{id}", web::get().to(search_doc))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
