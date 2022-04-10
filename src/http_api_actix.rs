
use std::sync::mpsc;

use actix_web::{
	rt::System,
	web::{self, Data},
	App,
	//	HttpRequest,
	HttpServer,
	Responder,
};

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
pub struct HttpApiActix {
	control_tx: 	mpsc::Sender< Message >,
	server_thread:  Option<std::thread::JoinHandle<()>>,
}

impl HttpApiActix {
	pub fn new( control_tx: mpsc::Sender< Message > ) -> Self {
		Self {
			control_tx,
			server_thread: None,
		}
	}

	pub fn run( &mut self ) -> anyhow::Result<()> {
		let tx = self.control_tx.clone();
		let server = HttpServer::new(move || {
			let http_state = HttpState {
				id:          "default".to_string(),
				http_sender: tx.clone(),
			};
			let http_state = Data::new(http_state);
			App::new()
				//									.data( http_state )
				.app_data(http_state)
				.route("/selectNextVariable", web::get().to(select_next_variable))
				.route(
					"/selectNextVariableWithPrefix/{prefix}",
					web::get().to(select_next_variable_with_prefix),
				)
				.route(
					"/incSelectedVariable/{value}",
					web::get().to(inc_selected_variable),
				)
				.route(
					"/decSelectedVariable/{value}",
					web::get().to(dec_selected_variable),
				)
				.route("/setVariable/{name}/{value}", web::get().to(set_variable))
				.route("/incVariable/{name}/{delta}", web::get().to(inc_variable))
				.route("/decVariable/{name}/{delta}", web::get().to(dec_variable))
				.route("/show/name/{name}", web::get().to(show_by_name))
				.route("/hide/name/{name}", web::get().to(hide_by_name))
				// :TODO: implement list_pages
				// :TODO: list element instances for specific page
				.route(
					"/list_element_instances",
					web::get().to(list_element_instances),
				)
				.route("/page/next", web::get().to(goto_next_page))
				.route("/page/prev", web::get().to(goto_prev_page))
				.route("/page/number/{number}", web::get().to(goto_page_number))
				.route("/page/name/{name}", web::get().to(goto_page_name))
		})
		.bind("0.0.0.0:8080")?
		.run();
		let server_thread = std::thread::spawn(move || {
			let sys = System::new(/*"test"*/);

			//				let _ = tx.send( server.clone() );

			match sys.block_on(server) {
				// :TODO: handle errors
				_ => {},
			}
		}); //.join().expect("Thread panicked");

		self.server_thread = Some(server_thread);
		Ok(())
	}
}

fn handle_response(rx: mpsc::Receiver<Response>) -> impl Responder {
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

async fn select_next_variable(state: web::Data<HttpState>) -> impl Responder {
	let (tx, rx) = std::sync::mpsc::channel();
	match state
		.http_sender
		.send(Message::SelectNextVariable(tx, None))
	{
		_ => {},
	};

	handle_response(rx)
}

async fn select_next_variable_with_prefix(
	state: web::Data<HttpState>,
	path: web::Path<String>,
) -> impl Responder {
	let prefix = path.into_inner();
	let (tx, rx) = std::sync::mpsc::channel();
	match state
		.http_sender
		.send(Message::SelectNextVariable(tx, Some(prefix)))
	{
		_ => {},
	};

	handle_response(rx)
}

async fn set_variable(
	state: web::Data<HttpState>,
	path: web::Path<(String, String)>,
) -> impl Responder {
	let (name, value) = path.into_inner();
	let (tx, rx) = std::sync::mpsc::channel();
	match state
		.http_sender
		.send(Message::SetVariable(tx, name.clone(), value.clone()))
	{
		_ => {},
	};

	handle_response(rx)
}

async fn inc_variable(
	state: web::Data<HttpState>,
	path: web::Path<(String, u32)>,
) -> impl Responder {
	let (name, delta) = path.into_inner();
	let (tx, rx) = std::sync::mpsc::channel();
	match state.http_sender.send(Message::IncrementVariable(
		tx,
		name.clone(),
		delta.try_into().unwrap(),
	)) {
		_ => {},
	};
	handle_response(rx)
}

async fn dec_variable(
	state: web::Data<HttpState>,
	path: web::Path<(String, u32)>,
) -> impl Responder {
	let (name, delta) = path.into_inner();
	let v: i32 = delta.try_into().unwrap();
	let (tx, rx) = std::sync::mpsc::channel();
	match state
		.http_sender
		.send(Message::IncrementVariable(tx, name.clone(), -v))
	{
		_ => {},
	};
	handle_response(rx)
}

async fn inc_selected_variable(
	state: web::Data<HttpState>,
	path: web::Path<u32>,
) -> impl Responder {
	let delta = path.into_inner();
	let (tx, rx) = std::sync::mpsc::channel();
	match state.http_sender.send(Message::IncrementSelectedVariable(
		tx,
		delta.try_into().unwrap(),
	)) {
		_ => {},
	};
	handle_response(rx)
}
async fn dec_selected_variable(
	state: web::Data<HttpState>,
	path: web::Path<u32>,
) -> impl Responder {
	let delta = path.into_inner();
	let v: i32 = delta.try_into().unwrap();
	let (tx, rx) = std::sync::mpsc::channel();
	match state
		.http_sender
		.send(Message::IncrementSelectedVariable(tx, -v))
	{
		_ => {},
	};
	handle_response(rx)
}

async fn show_by_name(state: web::Data<HttpState>, path: web::Path<String>) -> impl Responder {
	let name = path.into_inner();

	match state
		.http_sender
		.send(Message::SetElementVisibilityByName(name.clone(), true))
	{
		_ => {},
	};
	format!("show ({}) name == {}", &state.id, &name)
}

async fn hide_by_name(state: web::Data<HttpState>, path: web::Path<String>) -> impl Responder {
	let name = path.into_inner();

	match state
		.http_sender
		.send(Message::SetElementVisibilityByName(name.clone(), false))
	{
		_ => {},
	};
	format!("hide ({}) name == {}", &state.id, &name)
}

async fn list_element_instances(state: web::Data<HttpState>) -> impl Responder {
	let (sender, receiver) = mpsc::channel();

	match state
		.http_sender
		.send(Message::ListElementInstances(sender))
	{
		Ok(_) => {
			match receiver.recv() {
				Ok(msg) => match msg {
					Response::ElementInstanceList(l) => {
						return l;
					},
					_ => {
						dbg!(&msg);
					},
				},
				Err(e) => {
					dbg!(&e);
				},
			}
			//format!("elements ->")
			"elements ->".to_string()
		},
		_ => {
			//format!("{{}}")
			"{}".to_string()
		},
	}
}

async fn goto_next_page(state: web::Data<HttpState>) -> impl Responder {
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

	"{}".to_string()
}

async fn goto_prev_page(state: web::Data<HttpState>) -> impl Responder {
	let (sender, receiver) = mpsc::channel();
	match state.http_sender.send(Message::GotoPrevPage(sender)) {
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

async fn goto_page_number(state: web::Data<HttpState>, path: web::Path<usize>) -> impl Responder {
	let page_no = path.into_inner();
	let (sender, receiver) = mpsc::channel();
	match state.http_sender.send(Message::GotoPage(sender, page_no)) {
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

async fn goto_page_name(
	state: web::Data<HttpState>,
	path: web::Path<String>,
	//		web::Path( page_name ): web::Path< String >
) -> impl Responder {
	let page_name = path.into_inner();
	let (sender, receiver) = mpsc::channel();
	match state
		.http_sender
		.send(Message::GotoPageName(sender, page_name))
	{
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

