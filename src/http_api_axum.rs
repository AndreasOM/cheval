
use std::sync::mpsc;
use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use tracing::*;

use crate::control::{
	Message,
	Response,
};

#[derive(Debug)]
pub struct HttpApiAxum {
	control_tx: 	mpsc::Sender< Message >,
}

impl HttpApiAxum {
	pub fn new( control_tx: mpsc::Sender< Message > ) -> Self {
		Self {
			control_tx,
		}
	}

	pub async fn run( &self ) -> anyhow::Result<()> {
		let app = Router::new()
			.route("/page/next", get(goto_next_page))
		;
		let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
		let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

		Ok(())
	}
}

impl Drop for HttpApiAxum {
	fn drop(&mut self) { 
		debug!("Dropped HttpApiAxum");
	}
}

//async fn goto_next_page(state: web::Data<HttpState>) -> impl Responder {
async fn goto_next_page() -> &'static str {
	debug!("goto_next_page");
/*
	let (sender, receiver) = mpsc::channel();
	match state.http_sender.send(Message::GotoNextPage(sender)) {
		Ok(_) => match receiver.recv() {
			Ok(msg) => match msg {
				Response::PageChanged(new_page_no, old_page_no) => {
					return format!(
						r#"{{ "new_page":{}, "old_page":{} }}"#,
						new_page_no.unwrap_or(usize::MAX),
						old_page_no.unwrap_or(usize::MAX)
					);
				},
				_ => {
					dbg!(&msg);
				},
			},
			Err(e) => {
				dbg!(&e);
			},
		},
		_ => {},
	};
*/
	"{}" //.to_string()
}
