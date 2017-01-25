#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate iron;
extern crate env_logger;
extern crate bodyparser;
extern crate serde;
#[macro_use]
extern crate log;
extern crate ssh2;

mod execer;
use execer::sshexecer::SSHExecer;

use std::env;
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

fn exec(execer: execer::sshexecer::SSHExecer, req: &mut Request) -> IronResult<Response> {
    let exec_req = match req.get::<bodyparser::Struct<ExecRequest>>() {
        Ok(x) => x,
        Err(e) => {
            debug!("error unmarshalling: {}", e);
            return Err(IronError::new(Error {
                                          error: "invalid request; could not unmarshal body"
                                              .to_string(),
                                      },
                                      status::BadRequest));
        }
    };

    let exec_req = match exec_req {
        Some(er) => er,
        None => {
            return Err(IronError::new(Error {
                                          error: "invalid request; could not unmarshal body"
                                              .to_string(),
                                      },
                                      status::BadRequest));
        }
    };

    if exec_req.cmd_type != "cmd".to_string() {
        return Err(IronError::new(Error { error: "unrecognized cmdType".to_string() },
                                  status::BadRequest));
    }
    let er = match execer.exec(exec_req.exec, exec_req.body) {
        Ok((out, err, exit)) => {
            ExecResponse {
                stdout: out,
                stderr: err,
                exit_code: exit,
            }
        }
        Err(e) => {
            return Err(IronError::new(Error { error: "error execing cmd".to_string() },
                                      status::BadRequest));
        }
    };

    match serde_json::to_string(&er) {
        Err(e) => {
            warn!("error marshaling: {}", e);
            return Err(IronError::new(Error { error: "error marshalling response".to_string() },
                                      status::InternalServerError));
        }
        Ok(s) => {
            Ok(Response::with((status::Ok, s)))
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let _ = env::var("CMD_HOST").expect("CMD_HOST env variable must be set");
    let _ = env::var("CMD_USER").expect("CMD_USER env variable must be set");

    let chain = Chain::new(|req: &mut Request| -> IronResult<Response> {
        // TODO move out of closure
        let ssh_host = env::var("CMD_HOST").expect("CMD_HOST env variable must be set");
        let ssh_user = env::var("CMD_USER").expect("CMD_USER env variable must be set");
        let userauth_file = match env::var("CMD_PEM") {
            Ok(f) => Some(f),
            Err(_) => None,
        };
        let ssh_execer = SSHExecer::new(ssh_host, ssh_user, userauth_file);
        exec(ssh_execer, req)
    });
    Iron::new(chain).http("0.0.0.0:8080").unwrap();
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
