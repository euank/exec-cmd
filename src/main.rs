#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate iron;
extern crate bodyparser;
extern crate serde;
#[macro_use]
extern crate log;

use std::fmt;
use iron::prelude::*;
use iron::status;



// {"exec": "friendly-cmd-name", "body": "body-string", "type": "cmd"}
#[derive(Debug, Clone, Deserialize)]
struct ExecRequest {
    exec: String,
    #[serde(default)]
    body: String,
    #[serde(rename = "cmdType")]
    cmd_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct ExecResponse {
    stdout: Option<String>,
    stderr: Option<String>,
    #[serde(rename = "exitCode")]
    exit_code: i32,
}

fn exec(req: &mut Request) -> IronResult<Response> {
    let des: ExecRequest = match req.get::<bodyparser::Struct<ExecRequest>>() {
        Ok(x) => x,
        Err(e) => {
            debug!("error unmarshalling: {}", e);
            return Err(IronError::new(Error{
                error: "invalid request; could not unmarshal body".to_string(),
            }, status::BadRequest));
        },
    }.unwrap();
    let er = ExecResponse{
        stdout: None,
        stderr: None,
        exit_code: -1,
    };
    let mut result_buf: &mut [u8] = &mut [];
    match serde_json::to_writer(&mut result_buf, &er) {
        Err(e) => {
            warn!("error marshaling: {}", e);
            return Err(IronError::new(Error{
                error: "error marshalling response".to_string(),
            }, status::InternalServerError));
        },
        _ => {},
    };
    Ok(Response::with((status::Ok, result_buf as &[u8])))
}

fn main() {
    let chain = Chain::new(exec);
    Iron::new(chain).http(":8080").unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct Error {
    error: String,
}



impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let serialized = match serde_json::to_string(self) {
            Ok(s) => s,
            Err(_) => {
                return Err(fmt::Error);
            }
        };
        write!(f, "{}", serialized)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Error"
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}
