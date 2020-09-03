use crate::element::{Element, ElementConfig};
use crate::context::Context;

use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use async_trait::async_trait;

enum Message {
	FileChanged,
}

//#[derive(Debug)]
pub struct LoadTextElement {
	name: String,
	filename: String,
	variable: String,
	split_lines: bool,
	values: Vec<String>,
	watcher: Option<RecommendedWatcher>,
	channel: Option<Receiver<Message>>,
}

impl std::fmt::Debug for LoadTextElement {
	fn fmt( &self, f: &mut std::fmt::Formatter ) -> std::fmt::Result {
		writeln!( f,"LoadTextElement: :TODO:" )
	}
}

impl LoadTextElement {
}

unsafe impl Send for LoadTextElement {}

#[async_trait]
impl Element for LoadTextElement {
	fn configure( &mut self, config: &ElementConfig ) {
		self.filename = config.get_string_or( "filename", "" );
		self.variable = config.get_string_or( "variable", "" );
		self.split_lines = config.get_bool_or( "split_lines", false );
	}
	async fn run( &mut self ) -> anyhow::Result<()> {
		if self.filename != "" {
			let filename = self.filename.clone();
			let (tx, rx) = channel();

			let (tx2, rx2) = channel();
			tx2.send( Message::FileChanged ); // force update on start
			
			self.channel = Some( rx2 );

			let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
			watcher.watch(&self.filename, RecursiveMode::NonRecursive)?;

			self.watcher = Some( watcher );

			tokio::spawn(async move{
				loop {
		//				println!("LoadTextElement - run() {}", variable );
		//				tokio::time::delay_for( std::time::Duration::from_millis(5000) ).await;
					match rx.recv() {
						Ok(_event) => {
		//						println!("{:?}", event);
							tx2.send( Message::FileChanged );
						},
						Err(e) => {
							println!("watch error: {:?}", e);
		//						return Err( "".to_string() );	// :TODO:
						},
					}
		    	}
			});
		}

		Ok(())	
	}


	fn update( &mut self, context: &mut Context ) {
		if let Some( channel ) = &self.channel {
			match channel.try_recv() {
				Ok( _msg ) => {
					if let Ok( s ) = std::fs::read_to_string( &self.filename ) {
						// :TODO: format string
						if self.split_lines {
							let lines: Vec<&str> = s.split('\n').collect();
							println!("{:#?}", &lines);
							self.values = lines.iter().map( |l| l.to_string() ).collect();
							dbg!(&self.values);
						} else {
							println!("{:?}", &s);
							self.values[ 0 ] = s.to_string();
						}
					}
				},
				_ => {

				},
			}
		}
		if self.variable != "" {
			if self.split_lines {
				// :TODO: move into helper
				let parts : Vec<&str> = self.variable.split("{}").collect();
				let mut i = 0;
				for l in &self.values {
					let variable = match parts.len() {
						0 => format!("{}", i ),
						1 => format!("{}{}", parts[ 0 ], i ),
						2 => format!("{}{}{}", parts[ 0 ], i, parts[ 1 ] ),
						_ => format!("Variable template >{}< not supported", self.variable ), // panic?
					};
					context.set_string( &variable, &l );
					i += 1;
				}
			} else {
				context.set_string( &self.variable, &self.values[ 0 ] );
			}
		}
	}

	fn render( &self, buffer: &mut Vec<u32>, width: usize, height: usize ) {
	}
	fn name( &self ) -> &str {
		&self.name
	}
	fn set_name(&mut self, name: &str ) {
		self.name = name.to_string();
	}

	fn element_type( &self ) -> &str {
		"loadtext"
	}
}

pub struct LoadTextElementFactory {

}

impl LoadTextElementFactory {
	pub fn create() -> LoadTextElement {
		let mut values = Vec::new();
		values.push( "".to_string() );
		LoadTextElement {
			name: "".to_string(),
			filename: "".to_string(),
			variable: "".to_string(),
			split_lines: false,
			values: values,
			watcher: None,
			channel: None,
		}
	}
}

