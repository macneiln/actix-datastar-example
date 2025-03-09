use actix_sse_test::datastar_sse::DatastarSSE;
use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use async_stream::stream;
use datastar::prelude::*;
use maud::html;
use rand::Rng;
use std::{convert::Infallible, time::Duration};
use tokio::time::sleep;

async fn index() -> impl Responder {
    let index_page = html! {
        head {
            title {
                "Actix + Datastar Example"
            }
            script type="module" src="https://cdn.jsdelivr.net/gh/starfederation/datastar@v1.0.0-beta.9/bundles/datastar.js" {}
        }
        body {
            h2 {
                "Actix + Datastar Example"
            }
            div #count-down data-on-load="@get('/count-down')" {}
        }
    };
    HttpResponse::Ok().body(index_page.into_string())
}

async fn feed() -> impl Responder {
    let event_stream = stream! {
        let mut next_number = rand::rng().random_range(0..1_000);

        let fragment = html! {
            div #count-down {
                h3 {
                    "Count Down"
                }
                div {
                    span { "Starting Value: " (next_number) }
                }
                div {
                    span { "Current Value: "}
                    span data-signals-counter=(next_number) data-text="$counter" {}
                }
            }
        };

        let event: DatastarEvent = MergeFragments::new(fragment).into();
        yield Ok::<_, Infallible>(event);

        loop {

            sleep(Duration::from_millis(50)).await;
            next_number -= 1;
            let fragment = html! {
                "{ counter: " (next_number) " }"
            };
            let event: DatastarEvent = MergeSignals::new(fragment).into();
            yield Ok::<_, Infallible>(event);

            if next_number <= 0 {
                let fragment = html! {
                    "{ counter: 'Finished!' }"
                };
                let event: DatastarEvent = MergeSignals::new(fragment).into();
                yield Ok::<_, Infallible>(event);
                break;
            }
        }
    };

    DatastarSSE(event_stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = "127.0.0.1";
    let port = 8080;
    println!("Listening on: http://{}:{}", host, port);
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/count-down", web::get().to(feed))
    })
    .bind((host, port))?
    .run()
    .await
}
