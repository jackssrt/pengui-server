use std::{
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::{Result, bail};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    SinkExt, StreamExt, TryStreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{io::copy_buf, pin, sync::mpsc, task::AbortHandle};
use tokio_stream::wrappers::ReceiverStream;

const DELIMITER: char = '\u{FFFF}';

pub struct Connection {
    outgoing_sender: Arc<mpsc::Sender<Message>>,
    incoming_abort_handle: AbortHandle,
    outgoing_abort_handle: AbortHandle,
}
impl Connection {
    pub fn new(
        socket: WebSocket,
        incoming_handler: impl Fn(Box<[String]>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
        + Sync
        + Send
        + 'static,
    ) -> Self {
        let (sink, stream) = socket.split();
        let (outgoing_sender, outgoing_receiver) = mpsc::channel(100);
        let outgoing_sender = Arc::new(outgoing_sender);
        let internal_outgoing_sender = outgoing_sender.clone();
        Self {
            incoming_abort_handle: tokio::spawn(async move {
                Self::handle_incoming(stream, incoming_handler, internal_outgoing_sender).await;
            })
            .abort_handle(),

            outgoing_abort_handle: tokio::spawn(async move {
                Self::handle_outgoing(sink, outgoing_receiver);
            })
            .abort_handle(),
            outgoing_sender,
        }
    }
    pub async fn handle_incoming(
        mut stream: SplitStream<WebSocket>,
        handler: impl Fn(Box<[String]>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
        + Sync
        + Send
        + 'static,
        outgoing_sender: Arc<mpsc::Sender<Message>>,
    ) -> Result<()> {
        while let Some(value) = stream.try_next().await? {
            match value {
                Message::Binary(_) => bail!("unexpected binary message"),
                Message::Text(x) => {
                    let parts: Box<[String]> = x.split(DELIMITER).map(ToOwned::to_owned).collect();
                    handler(parts).await
                }
                Message::Close(_) => {
                    outgoing_sender.send(Message::Close(None)).await;
                    Ok(())
                }
                _ => Ok(()),
            }?
        }
        Ok(())
    }
    pub async fn handle_outgoing(
        mut sink: SplitSink<WebSocket, Message>,
        mut receiver: mpsc::Receiver<Message>,
    ) {
        while let Some(value) = receiver.recv().await {
            sink.send(value);
        }
    }
}
