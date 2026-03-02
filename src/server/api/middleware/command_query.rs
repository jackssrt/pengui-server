use axum::{
    extract::{FromRequestParts, Query},
    http::Request,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct CommandQuery {
    command: String,
}

pub fn rewrite_command_query<B>(mut req: Request<B>) -> Request<B> {
    let query = Query::<CommandQuery>::try_from_uri(req.uri());
    if let Ok(Query(CommandQuery { command })) = query {
        let uri = req.uri_mut();
        let new_uri = format!(
            "{}/{}?{}",
            uri.path(),
            command,
            // already exists, the Query extractor checked it
            uri.query().unwrap()
        )
        .parse();
        debug_assert!(new_uri.is_ok(), "Failed to rewrite command query to path");
        if let Ok(new_uri) = new_uri {
            // rewrite the request
            *uri = new_uri;
        }
        // fail silently, probably leaving the request as a 404
    }

    req
}
