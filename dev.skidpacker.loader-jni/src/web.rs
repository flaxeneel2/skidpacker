use actix_web::{App, HttpServer, web};

pub struct WebServer {

}

impl WebServer {
    async fn init(&self) {
        HttpServer::new(|| {
            App::new()
                .route("/hello", web::get().to(|| async { "Hello World!" }))
        })
            .bind(("127.0.0.1", 8080)).unwrap()
            .run()
            .await.expect("Failed to launch webserver!");


    }
}