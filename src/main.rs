use actix_web::{
    http::{header::CACHE_CONTROL, StatusCode},
    web::{self, Bytes},
    App, HttpResponse, HttpResponseBuilder, HttpServer, Responder,
};
use async_stream::stream;
use datastar::{prelude::MergeFragments, DatastarEvent};
use maud::html;
use rand::Rng;
use std::{convert::Infallible, time::Duration};
use tokio::time::sleep;

async fn index() -> impl Responder {
    let index_page = r#"
    <!doctype html><html>
      <head>
        <title>Actix + Datastar Example</title>
        <script type="module" src="https://cdn.jsdelivr.net/gh/starfederation/datastar@v1.0.0-beta.9/bundles/datastar.js"></script>
      </head>
      <body>
        <h2>Actix + Datastar Example</h2>
        <div>
          <span>Single server response: </span>
          <span id="once" data-on-load="@get('/once')"></span>
        </div>
        <div>
          <span>Continuous server feed: </span>
          <span id="feed" data-on-load="@get('/feed')"></span>
        </div>
      </body>
    </html>"#;
    HttpResponse::Ok().body(index_page)
}

async fn feed() -> impl Responder {
    let response_stream = stream! {
        let mut next_number = rand::rng().random_range(0..1_000_000);

        loop {
            let fragment = html! {
                span #feed {
                    (next_number)
                }
            };

            let event: DatastarEvent = MergeFragments::new(fragment).into();
            yield Ok::<_, Infallible>(Bytes::from(event.to_string()));
            sleep(Duration::from_millis(100)).await;
            next_number -= 1;

        }
    };

    HttpResponseBuilder::new(StatusCode::OK)
        .content_type("text/event-stream")
        .keep_alive()
        .append_header((CACHE_CONTROL, "no-cache"))
        .streaming(response_stream)
}

async fn once() -> impl Responder {
    let response_stream = stream! {
        let next_number = rand::rng().random_range(0..1_000_000);

            let fragment = html! {
                span #once {
                    (next_number)
                }
            };

            let event: DatastarEvent = MergeFragments::new(fragment).into();
            yield Ok::<_, Infallible>(Bytes::from(event.to_string()));
    };

    HttpResponseBuilder::new(StatusCode::OK)
        .content_type("text/event-stream")
        .keep_alive()
        .append_header((CACHE_CONTROL, "no-cache"))
        .streaming(response_stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = "127.0.0.1";
    let port = 8080;
    println!("Listening on: http://{}:{}", host, port);
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/feed", web::get().to(feed))
            .route("/once", web::get().to(once))
    })
    .bind((host, port))?
    .run()
    .await
}
