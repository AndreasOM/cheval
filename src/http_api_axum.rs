
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
			.route("/selectNextVariable", get(select_next_variable))
			.route(
				"/selectNextVariableWithPrefix/:prefix",
				get(select_next_variable_with_prefix),
			)
			.route(
				"/incSelectedVariable/:value",
				get(inc_selected_variable),
			)
			.route(
				"/decSelectedVariable/:value",
				get(dec_selected_variable),
			)
			.route("/setVariable/:name/:value", get(set_variable))
			.route("/incVariable/:name/:delta", get(inc_variable))
			.route("/decVariable/:name/:delta", get(dec_variable))
			.layer(Extension(http_state))
		;
		let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
		let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

        debug!("done");
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

fn handle_response(rx: mpsc::Receiver<Response>) -> String {
	match rx.recv() {
		Ok(r) => {
			match r {
				Response::VariableSelected(name) => {
					format!("variable selected: {}", &name) // :TODO: decide on formatting
				},
				Response::VariableChanged(name, v) => {
					format!("{{\"variables\":[{{ \"{}\": {}}}]}}", &name, v)
				},
				Response::VariableU32Changed(name, v) => {
					format!("{{\"variables\":[{{ \"{}\": {}}}]}}", &name, v)
				},
				Response::VariableF32Changed(name, v) => {
					format!("{{\"variables\":[{{ \"{}\": {}}}]}}", &name, v)
				},
				Response::VariableStringChanged(name, v) => {
					format!("{{\"variables\":[{{ \"{}\": \"{}\"}}]}}", &name, &v)
				},
				o => {
					format!("Unhandled response: {:?}", &o) // :TODO: format as json
				},
			}
		},
		Err(e) => format!("Error: {:?}", &e), // :TODO: format as json
	}
}

fn send_message_and_handle_response( state: &Arc<std::sync::Mutex<HttpState>>, message: Message, receiver: mpsc::Receiver< Response > ) -> impl IntoResponse {
	let state = state.lock().unwrap();
	match state.http_sender.send(message) {
		Ok(_) => handle_response( receiver ),
		_ => "{}".to_string(),
	}
}


async fn select_next_variable(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
) -> impl IntoResponse {
	debug!("select_next_variable" );
	let (sender, receiver) = mpsc::channel();
	send_message_and_handle_response( &state, Message::SelectNextVariable(sender, None ), receiver )
}

async fn select_next_variable_with_prefix(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path( prefix ): Path<String>,
) -> impl IntoResponse {
	debug!("select_next_variable_with_prefix {}", &prefix );
	let (sender, receiver) = mpsc::channel();
	send_message_and_handle_response( &state, Message::SelectNextVariable(sender, Some(prefix.clone()) ), receiver )
}

async fn set_variable(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path((name,value)): Path<(String,String)>,
) -> impl IntoResponse {
	debug!("set_variable {} => {}", &name, &value );
	let (sender, receiver) = mpsc::channel();
	send_message_and_handle_response( &state, Message::SetVariable(sender, name.clone(), value.clone() ), receiver )
}

async fn inc_variable(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path((name,delta)): Path<(String,i32)>,
) -> impl IntoResponse {
	debug!("inc_variable {} +{}", &name, &delta );
	let (sender, receiver) = mpsc::channel();
	send_message_and_handle_response( &state, Message::IncrementVariable(sender, name.clone(), delta ), receiver )
}

async fn dec_variable(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path((name,delta)): Path<(String,i32)>,
) -> impl IntoResponse {
	debug!("dec_variable {} -{}", &name, &delta );
	let (sender, receiver) = mpsc::channel();
	send_message_and_handle_response( &state, Message::IncrementVariable(sender, name.clone(), -delta ), receiver )
}

async fn inc_selected_variable(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path(delta): Path<i32>,
) -> impl IntoResponse {
	debug!("inc_selected_variable +{}", &delta );
	let (sender, receiver) = mpsc::channel();
	send_message_and_handle_response( &state, Message::IncrementSelectedVariable(sender, delta ), receiver )
}

async fn dec_selected_variable(
	Extension(state): Extension<Arc<std::sync::Mutex<HttpState>>>,
	Path(delta): Path<i32>,
) -> impl IntoResponse {
	debug!("dec_selected_variable -{}", &delta );
	let (sender, receiver) = mpsc::channel();
	send_message_and_handle_response( &state, Message::IncrementSelectedVariable(sender, -delta ), receiver )
}
