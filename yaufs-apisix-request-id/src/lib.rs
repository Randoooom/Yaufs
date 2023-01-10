#[macro_use]
extern crate log;

use proxy_wasm::traits::*;
use proxy_wasm::types::*;

const ALPHABET: [char; 16] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(RequestIdRoot) });
}}

struct RequestIdRoot;
struct RequestId;

impl Context for RequestIdRoot {}
impl Context for RequestId {}

impl RootContext for RequestIdRoot {
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(RequestId))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

impl HttpContext for RequestId {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let id = nanoid::nanoid!(32, &ALPHABET);
        debug!("Assigning X-Request-Id {} to incoming request", id.as_str());
        self.add_http_request_header("X-Request-Id", id.as_str());
        self.add_http_response_header("X-Request-Id", id.as_str());

        Action::Continue
    }
}
