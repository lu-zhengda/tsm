use std::cell::RefCell;

use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
use ureq::Agent;

use crate::config::Config;
use crate::error::Error;
use crate::rpc::types::{LegacyRpcResponse, RpcRequest};

pub struct TransmissionClient {
    url: String,
    auth: Option<(String, String)>,
    session_id: RefCell<Option<String>>,
    agent: Agent,
}

fn validate_host(host: &str) -> Result<(), Error> {
    if host.is_empty() {
        return Err(Error::Config("Host cannot be empty".to_string()));
    }
    if host.contains(['/', '@', '#', '?', ' ']) {
        return Err(Error::Config(format!(
            "Invalid host: '{host}' contains disallowed characters"
        )));
    }
    Ok(())
}

impl TransmissionClient {
    pub fn new(config: &Config) -> Result<Self, Error> {
        validate_host(&config.host)?;

        let scheme = if config.port == 443 { "https" } else { "http" };
        let url = format!(
            "{scheme}://{}:{}/transmission/rpc",
            config.host, config.port
        );
        let auth = match (&config.username, &config.password) {
            (Some(u), Some(p)) => Some((u.clone(), p.clone())),
            (None, None) => None,
            (Some(_), None) => {
                return Err(Error::Config(
                    "Username provided without password. Set TSM_PASSWORD or use --password."
                        .to_string(),
                ));
            }
            (None, Some(_)) => {
                return Err(Error::Config(
                    "Password provided without username. Set TSM_USERNAME or use --username."
                        .to_string(),
                ));
            }
        };

        let agent: Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();

        Ok(Self {
            url,
            auth,
            session_id: RefCell::new(None),
            agent,
        })
    }

    pub fn request(&self, method: &str, params: Option<Value>) -> Result<Value, Error> {
        // First attempt
        let result = self.do_request(method, &params);

        match result {
            Ok(val) => Ok(val),
            Err(Error::SessionExpired) => {
                // Session ID was extracted during the 409 handling, retry
                self.do_request(method, &params).map_err(|e| match e {
                    Error::SessionExpired => Error::Connection(
                        "Server rejected session token twice. Try again.".to_string(),
                    ),
                    other => other,
                })
            }
            Err(e) => Err(e),
        }
    }

    fn do_request(&self, method: &str, params: &Option<Value>) -> Result<Value, Error> {
        let rpc_request = RpcRequest::new(method, params.clone());
        let body = serde_json::to_string(&rpc_request)
            .map_err(|e| Error::Rpc(format!("Failed to serialize request: {e}")))?;

        let mut req = self
            .agent
            .post(&self.url)
            .header("Content-Type", "application/json");

        if let Some(session_id) = self.session_id.borrow().as_ref() {
            req = req.header("X-Transmission-Session-Id", session_id);
        }

        if let Some((username, password)) = &self.auth {
            let credentials = format!("{username}:{password}");
            let encoded = STANDARD.encode(credentials.as_bytes());
            req = req.header("Authorization", &format!("Basic {encoded}"));
        }

        let response = req.send(body.as_bytes()).map_err(|e| {
            Error::Connection(format!(
                "Could not connect to Transmission at {}. Is the daemon running?\nDetails: {e}",
                self.url
            ))
        })?;

        let status = response.status();

        // Handle 409: extract session ID and signal retry
        if status == 409 {
            if let Some(sid) = response.headers().get("X-Transmission-Session-Id") {
                let sid_str = sid.to_str().map_err(|_| {
                    Error::Connection("Invalid session ID header from server".to_string())
                })?;
                *self.session_id.borrow_mut() = Some(sid_str.to_string());
            }
            return Err(Error::SessionExpired);
        }

        if status == 401 {
            return Err(Error::Auth);
        }

        if status != 200 {
            return Err(Error::Rpc(format!("Unexpected HTTP status: {status}")));
        }

        // Update session ID if present in successful response
        if let Some(sid) = response.headers().get("X-Transmission-Session-Id")
            && let Ok(sid_str) = sid.to_str()
        {
            *self.session_id.borrow_mut() = Some(sid_str.to_string());
        }

        let response_body = response
            .into_body()
            .read_to_string()
            .map_err(|e| Error::Rpc(format!("Failed to read response: {e}")))?;

        // Parse legacy format: {"result": "success", "arguments": {...}}
        let legacy: LegacyRpcResponse = serde_json::from_str(&response_body)
            .map_err(|e| Error::Rpc(format!("Failed to parse response: {e}")))?;

        if let Some(result_str) = &legacy.result
            && result_str != "success"
        {
            return Err(Error::Rpc(result_str.clone()));
        }

        // Return arguments, or empty object for commands with no return data
        Ok(legacy
            .arguments
            .unwrap_or(Value::Object(serde_json::Map::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_encode() {
        let encoded = STANDARD.encode(b"user:pass");
        assert_eq!(encoded, "dXNlcjpwYXNz");
    }

    #[test]
    fn test_client_new() {
        let config = Config {
            host: "myhost".to_string(),
            port: 1234,
            username: Some("admin".to_string()),
            password: Some("secret".to_string()),
            json: false,
        };

        let client = TransmissionClient::new(&config).unwrap();
        assert_eq!(client.url, "http://myhost:1234/transmission/rpc");
        assert!(client.auth.is_some());
    }

    #[test]
    fn test_client_https_on_443() {
        let config = Config {
            host: "myhost".to_string(),
            port: 443,
            username: None,
            password: None,
            json: false,
        };

        let client = TransmissionClient::new(&config).unwrap();
        assert_eq!(client.url, "https://myhost:443/transmission/rpc");
    }

    #[test]
    fn test_client_no_auth() {
        let config = Config {
            host: "localhost".to_string(),
            port: 9091,
            username: None,
            password: None,
            json: false,
        };

        let client = TransmissionClient::new(&config).unwrap();
        assert_eq!(client.url, "http://localhost:9091/transmission/rpc");
        assert!(client.auth.is_none());
    }

    #[test]
    fn test_partial_credentials_rejected() {
        let config = Config {
            host: "localhost".to_string(),
            port: 9091,
            username: Some("admin".to_string()),
            password: None,
            json: false,
        };

        assert!(TransmissionClient::new(&config).is_err());

        let config2 = Config {
            host: "localhost".to_string(),
            port: 9091,
            username: None,
            password: Some("secret".to_string()),
            json: false,
        };

        assert!(TransmissionClient::new(&config2).is_err());
    }

    #[test]
    fn test_invalid_host_rejected() {
        let config = Config {
            host: "evil.com/foo#".to_string(),
            port: 9091,
            username: None,
            password: None,
            json: false,
        };

        assert!(TransmissionClient::new(&config).is_err());
    }
}
