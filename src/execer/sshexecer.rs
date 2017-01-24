use ssh2::Session;
use std::error::Error;
use std::net::TcpStream;
use std::fmt;
use std::io::Read;

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

    pub fn exec(&self,
                namespace: String,
                cmd: String)
                -> Result<(Option<String>, Option<String>, i32), SSHExecErrors> {
        let tcp = match TcpStream::connect::<&str>(self.host.as_ref()) {
            Ok(x) => x,
            Err(_) => {
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

        match sess.userauth_agent(&self.user) {
            Err(_) => {
                return Err(SSHExecErrors::SessionCreationError);
            }
            _ => {}
        };


        let mut channel = match sess.channel_session() {
            Err(_) => {
                return Err(SSHExecErrors::SessionCreationError);
            }
            Ok(x) => x,
        };

        match channel.exec(&format!("{} {}", namespace, cmd)) {
            Err(_) => {
                return Err(SSHExecErrors::ChannelExecError);
            }
            _ => {}
        };
        let mut output = String::new();
        channel.read_to_string(&mut output).unwrap();
        // TODO channel.stderr
        let exit_code = match channel.exit_status() {
            Err(_) => -1,
            Ok(e) => e,
        };

        Ok((Some(output), None, exit_code))
    }
}

#[derive(Debug,Clone)]
pub enum SSHExecErrors {
    TCPConnectError,
    SessionCreationError,
    HandshakeError,
    ChannelExecError,
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
            SSHExecErrors::ChannelExecError => "channel exec error",
        }
    }
}

impl fmt::Display for SSHExecErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}
