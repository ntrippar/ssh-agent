use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

use std::os::unix::net::{UnixListener, UnixStream};

use protocol::Request;

use handler::SSHAgentHandler;

use error::HandleResult;
pub struct Agent;

impl Agent {

	fn handle_client<T: SSHAgentHandler>(handler: Arc<Mutex<T>>, mut stream: UnixStream) -> HandleResult<()> {
		debug!("handling new connection");
		loop {
			let req = Request::read(&mut stream)?;
			debug!("request: {:?}", req);
			let response = handler.lock().unwrap().handle_request(req)?;
			debug!("handler: {:?}", response);
			response.write(&mut stream)?;
		}

	}

	pub fn run<T:SSHAgentHandler + 'static>(handler: T, listener: UnixListener) {
		let arc_handler = Arc::new(Mutex::new(handler));
		// accept the connections and spawn a new thread for each one 
		for stream in listener.incoming() {
			match stream {
				Ok(stream) => {
					// connection succeded
					let ref_handler = arc_handler.clone();
					thread::spawn( ||{
						match Agent::handle_client(ref_handler, stream){
							Ok(_) => {},
							Err(e) => debug!("handler: {:?}", e),
						}
					});
				}
				Err(_) => {
					// connection failed
					break;
				}
			}
		}
	}
}