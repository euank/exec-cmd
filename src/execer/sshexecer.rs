use ssh2::Session;
use std::error::Error;
use std::net::TcpStream;
use std::fmt;

// TODO tcp connection pool
pub struct SSHExecer {
    host: String,
    user: String,
}

impl SSHExecer {
    pub fn new(host: String, user: String) -> Self {
        SSHExecer {
            host: host,
            user: user,
        }
    }

    pub fn exec(&self, namespace: String, cmd: String) -> Result<(Option<String>, Option<String>, i32), SSHExecErrors> {
        let tcp = match TcpStream::connect::<&str>(self.host.as_ref()) {
            Ok(x) => x,
            Err(e) => {
                return Err(SSHExecErrors::TCPConnectError);
            }
        };
        let mut sess = match Session::new() {
            Some(s) => s,
            None => {
                return Err(SSHExecErrors::SessionCreationError);
            }
        };
        match sess.handshake(&tcp) {
            Err(_) => {
                return Err(SSHExecErrors::HandshakeError);
            }
            _ => {}
        };

        println!("TODO: I would do the ssh exec thing with {}", cmd);

        Ok((None, None, 0))
    }
}

#[derive(Debug,Clone)]
enum SSHExecErrors {
    TCPConnectError,
    SessionCreationError,
    HandshakeError,
}

impl Error for SSHExecErrors {
    fn cause(&self) -> Option<&Error> {
        None
    }
    fn description(&self) -> &str {
        match *self {
            SSHExecErrors::TCPConnectError => "tcp connect error",
            SSHExecErrors::SessionCreationError => "ssh session creation error",
            SSHExecErrors::HandshakeError => "ssh handshake error",
        }
    }
}

impl fmt::Display for SSHExecErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}
