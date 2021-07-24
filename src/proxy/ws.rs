use actix::{Actor, AsyncContext, StreamHandler};
use actix_http::{client::ConnectionIo, Uri};
use actix_web::web;
use actix_web_actors::ws;
use awc::Client;
use futures::{SinkExt, StreamExt};

use crate::config::SharedConfig;

use super::ServerError;

struct WsProxy {
    upstream_stream: actix_codec::Framed<Box<dyn ConnectionIo>, actix_http::ws::Codec>,
}

impl Actor for WsProxy {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsProxy {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match dbg!(msg) {
            Ok(msg) => ctx.spawn(async {
                self.upstream_stream.send(msg).await;
            }),
            _ => (),
        }
    }
}

pub(super) async fn begin(
    req: actix_web::HttpRequest,
    uri: Uri,
    client: &Client,
    config: &SharedConfig,
    stream: web::Payload,
) -> Result<actix_web::HttpResponse, ServerError> {
    let (_resp, upstream_stream) = client
        .ws(uri)
        .connect()
        .await
        .map_err(ServerError::WebsocketUpstreamHandshake)?;

    let (tx, rx) = tokio::sync::mpsc::channel(10);

    tokio::task::spawn_local(async {
        let next = upstream_stream.next().await;
    });

    let resp = ws::start(WsProxy { upstream_stream }, &req, stream)
        .map_err(ServerError::WebsocketClientHandshake)?;

    Ok(resp)
}
