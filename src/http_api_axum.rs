
use std::sync::{
	Arc,
	mpsc,
};
use std::net::SocketAddr;

use axum::{
	Extension,
	extract::Path,
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
struct HttpState {
	id:          String,
	http_sender: mpsc::Sender<Message>,
}

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
		let http_state = Arc::new(
			std::sync::Mutex::new(
				HttpState {
					id:          "default".to_string(),
					http_sender: self.control_tx.clone(),
				}
			)
		);
		let app = Router::new()
			.route("/page/next", get(goto_next_page))
			.route("/page/prev", get(goto_prev_page))
			.route("/page/number/:page_no", get(goto_page_number))
			.route("/page/name/:page_name", get(goto_page_name))
			.route("/show/name/:name", get(show_by_name))
			.route("/hide/name/:name", get(hide_by_name))
			.layer(Extension(http_state))
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

fn change_page( state: &Arc<std::sync::Mutex<HttpState>>, message: Message, receiver: mpsc::Receiver< Response > ) -> impl IntoResponse {
	let state = state.lock().unwrap();
	match state.http_sender.send(message) {
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
	"{}".to_string()
}

async fn goto_next_page(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
) -> impl IntoResponse {
	debug!("goto_next_page");
	let (sender, receiver) = mpsc::channel();
	change_page( &state, Message::GotoNextPage(sender), receiver )
}

async fn goto_prev_page(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
) -> impl IntoResponse {
	debug!("goto_prev_page");
	let (sender, receiver) = mpsc::channel();
	change_page( &state, Message::GotoPrevPage(sender), receiver )
}

async fn goto_page_number(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path(page_no): Path<usize>
) -> impl IntoResponse {
	debug!("goto_page_number {}", page_no);
	let (sender, receiver) = mpsc::channel();
	change_page( &state, Message::GotoPage(sender, page_no), receiver )
}

async fn goto_page_name(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path(page_name): Path<String>
) -> impl IntoResponse {
	debug!("goto_page_name {}", page_name);
	let (sender, receiver) = mpsc::channel();
	change_page( &state, Message::GotoPageName(sender, page_name), receiver )
}

async fn show_by_name(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path(name): Path<String>
) -> impl IntoResponse {
	let state = state.lock().unwrap();
	match state
		.http_sender
		.send(Message::SetElementVisibilityByName(name.clone(), true))
	{
		_ => {},
	};
	format!("show ({}) name == {}", &state.id, &name)
}

async fn hide_by_name(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path(name): Path<String>
) -> impl IntoResponse {
	let state = state.lock().unwrap();
	match state
		.http_sender
		.send(Message::SetElementVisibilityByName(name.clone(), false))
	{
		_ => {},
	};
	format!("hide ({}) name == {}", &state.id, &name)
}

