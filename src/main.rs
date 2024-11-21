use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web_lab::sse::{self};
use futures_util::stream;
use rand::Rng;
use std::{convert::Infallible, time::Duration};
use tokio::time::sleep;

async fn index() -> impl Responder {
    let index_page = r#"
    <!doctype html><html>
      <head>
        <title>Actix + Datastar Example</title>
        <script type="module" defer src="https://cdn.jsdelivr.net/npm/@sudodevnull/datastar"></script></head>
      <body>
        <h2>Actix + Datastar Example</h2>
        <div>
          <span>Single server response: </span>
          <span id="once" data-on-load="$$get('/once')"></span>
        </div>
        <div>
          <span>Continuous server feed: </span>
          <span id="feed" data-on-load="$$get('/feed')"></span>
        </div>
      </body>
    </html>"#;
    HttpResponse::Ok().body(index_page)
}

fn send_sse(fragment: &str, selector: Option<&str>, merge_type: Option<&str>) -> String {
    let mut response = String::new();

    if let Some(selector) = selector {
        response.push_str(&format!("selector {}\n", selector));
    }
    if let Some(merge_type) = merge_type {
        response.push_str(&format!("merge {}\n", merge_type));
    }
    response.push_str(&format!("fragment {}\n\n", fragment));
    response
}

async fn feed() -> impl Responder {
    let response_stream = stream::unfold((false, None), |(started, number)| async move {
        if started {
            sleep(Duration::from_millis(100)).await;
        }

        let next_number = if let Some(number) = number {
            number
        } else {
            rand::thread_rng().gen_range(0..1_000_000)
        };

        let fragment = &format!("<span id=\"feed\">{}</span>", next_number);
        let data = sse::Data::new(send_sse(fragment, None, None)).event("datastar-fragment");

        Some((
            Ok::<_, Infallible>(sse::Event::Data(data)),
            (true, Some(next_number - 1)),
        ))
    });

    sse::Sse::from_stream(response_stream).with_keep_alive(Duration::from_secs(60 * 5))
}

async fn once() -> impl Responder {
    let response_stream = stream::once(async move {
        let num = rand::thread_rng().gen_range(0..1_000_000);
        let fragment = &format!("<span id=\"once\">{}</span>", num);
        let data = sse::Data::new(send_sse(fragment, None, None)).event("datastar-fragment");

        Ok::<_, Infallible>(sse::Event::Data(data))
    });

    sse::Sse::from_stream(response_stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/feed", web::get().to(feed))
            .route("/once", web::get().to(once))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
