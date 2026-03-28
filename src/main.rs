#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::unused_async)]
#![allow(clippy::significant_drop_tightening)]
#![forbid(unused_must_use)]
#![feature(sync_nonpoison)]
#![feature(nonpoison_rwlock)]
#![feature(nonpoison_mutex)]
#![feature(duration_constructors)]
#![feature(stmt_expr_attributes)]
#![feature(lock_value_accessors)]
#![deny(clippy::panic)]
use anyhow::Result;

use crate::server::start;

mod chat;
mod client;
mod locations;
mod party;
mod player;
mod room;
mod server;
mod session;

#[tokio::main]
async fn main() -> Result<()> {
    start().await
}
