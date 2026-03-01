#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(dead_code)]
#![feature(sync_nonpoison)]
#![feature(nonpoison_rwlock)]
#![feature(nonpoison_mutex)]
#![feature(duration_constructors)]
use anyhow::Result;

use crate::server::start;
mod chat;
mod connection;
mod party;
mod player;
mod room;
mod server;
mod session;

#[tokio::main]
async fn main() -> Result<()> {
    start().await
}
