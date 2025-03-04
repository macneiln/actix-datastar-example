use actix_web::{
    body::BoxBody,
    http::{
        header::{CacheControl, CacheDirective},
        StatusCode,
    },
    web::Bytes,
    HttpResponse, HttpResponseBuilder, Responder,
};
use async_stream::stream;
use datastar::DatastarEvent;
use futures_util::Stream;
pub struct DatastarSSE<S, E>(pub S)
where
    S: Stream<Item = Result<DatastarEvent, E>> + 'static,
    E: Into<Box<dyn std::error::Error>> + 'static;

impl<S, E> Responder for DatastarSSE<S, E>
where
    S: Stream<Item = Result<DatastarEvent, E>> + 'static,
    E: Into<Box<dyn std::error::Error>> + 'static,
{
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let stream = stream! {
            for await result in self.0 {
                if let Ok(event) = result {
                    yield Ok(Bytes::from(event.to_string()));
                }
                else {
                    yield Err(result.unwrap_err());
                }
            }
        };

        HttpResponseBuilder::new(StatusCode::OK)
            .content_type(mime::TEXT_EVENT_STREAM)
            .insert_header(CacheControl(vec![CacheDirective::NoCache]))
            .keep_alive()
            .streaming(stream)
    }
}
