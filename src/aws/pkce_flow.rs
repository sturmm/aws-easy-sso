use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use url::Url;

const CALLBACK_PORT: u16 = 55200;
const CALLBACK_PATH: &str = "/oauth/callback";
pub const REDIRECT_URI: &str = "http://127.0.0.1:55200/oauth/callback";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REQUEST_SIZE: usize = 16 * 1024;

pub struct LocalServer {
    listener: TcpListener,
    pub redirect_uri: String,
}

enum Callback {
    Code(String),
    AuthorizationError(String),
}

impl LocalServer {
    pub async fn bind() -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", CALLBACK_PORT)).await?;
        Ok(Self {
            listener,
            redirect_uri: REDIRECT_URI.to_string(),
        })
    }

    pub async fn wait_for_code(self, expected_state: &str) -> Result<String> {
        timeout(LOGIN_TIMEOUT, async {
            loop {
                let (mut socket, _) = self.listener.accept().await?;
                match timeout(
                    REQUEST_TIMEOUT,
                    Self::read_callback(&mut socket, expected_state),
                )
                .await
                {
                    Ok(Ok(Callback::Code(code))) => {
                        Self::respond(&mut socket, "200 OK", SUCCESS_PAGE).await?;
                        return Ok(code);
                    }
                    Ok(Ok(Callback::AuthorizationError(error))) => {
                        Self::respond(&mut socket, "400 Bad Request", ERROR_PAGE).await?;
                        return Err(anyhow!("Authorization was rejected: {error}"));
                    }
                    Ok(Err(error)) => {
                        Self::respond(&mut socket, "400 Bad Request", INVALID_CALLBACK_PAGE)
                            .await?;
                        eprintln!("Ignoring invalid OAuth callback: {error}");
                    }
                    Err(_) => {
                        Self::respond(&mut socket, "408 Request Timeout", INVALID_CALLBACK_PAGE)
                            .await?;
                        eprintln!("Ignoring timed-out OAuth callback request");
                    }
                }
            }
        })
        .await
        .map_err(|_| anyhow!("Timed out waiting for the browser authorization callback"))?
    }

    async fn read_callback(socket: &mut TcpStream, expected_state: &str) -> Result<Callback> {
        let mut request = Vec::with_capacity(1024);
        let mut chunk = [0u8; 1024];

        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            if request.len() >= MAX_REQUEST_SIZE {
                return Err(anyhow!(
                    "OAuth callback request headers exceeded {MAX_REQUEST_SIZE} bytes"
                ));
            }

            let bytes_read = socket.read(&mut chunk).await?;
            if bytes_read == 0 {
                return Err(anyhow!(
                    "Connection closed before receiving an OAuth callback"
                ));
            }
            request.extend_from_slice(&chunk[..bytes_read]);
        }

        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut parsed_request = httparse::Request::new(&mut headers);
        if !matches!(
            parsed_request.parse(&request)?,
            httparse::Status::Complete(_)
        ) {
            return Err(anyhow!("Incomplete OAuth callback request"));
        }

        if parsed_request.method != Some("GET") {
            return Err(anyhow!("OAuth callback must use GET"));
        }

        let path = parsed_request
            .path
            .ok_or_else(|| anyhow!("OAuth callback has no path"))?;
        let url = Url::parse(&format!("http://localhost{path}"))?;
        if url.path() != CALLBACK_PATH {
            return Err(anyhow!("OAuth callback used an unexpected path"));
        }

        let query_pairs: HashMap<_, _> = url.query_pairs().into_owned().collect();
        let state = query_pairs
            .get("state")
            .ok_or_else(|| anyhow!("OAuth callback has no state"))?;
        if state != expected_state {
            return Err(anyhow!("OAuth callback has an unexpected state"));
        }

        if let Some(error) = query_pairs.get("error") {
            let description = query_pairs
                .get("error_description")
                .map(String::as_str)
                .unwrap_or("No additional details were provided");
            return Ok(Callback::AuthorizationError(format!(
                "{error}: {description}"
            )));
        }

        let code = query_pairs
            .get("code")
            .ok_or_else(|| anyhow!("OAuth callback has no authorization code"))?
            .to_string();
        Ok(Callback::Code(code))
    }

    async fn respond(socket: &mut TcpStream, status: &str, body: &str) -> Result<()> {
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await?;
        socket.shutdown().await?;
        Ok(())
    }
}

const SUCCESS_PAGE: &str = "<!doctype html><title>Login successful</title><p>Login successful. You can close this window and return to the CLI.</p>";
const ERROR_PAGE: &str = "<!doctype html><title>Login cancelled</title><p>Authorization was cancelled or rejected. You can close this window.</p>";
const INVALID_CALLBACK_PAGE: &str = "<!doctype html><title>Invalid callback</title><p>This callback is not valid. You can close this window.</p>";

#[cfg(test)]
mod tests {
    use super::*;

    async fn send_callback(path: &str) {
        let mut socket = TcpStream::connect(("127.0.0.1", CALLBACK_PORT))
            .await
            .unwrap();
        let request = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        socket.write_all(request.as_bytes()).await.unwrap();
        socket.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn ignores_invalid_requests_until_the_valid_callback_arrives() {
        let server = LocalServer::bind().await.unwrap();
        let wait_for_code =
            tokio::spawn(async move { server.wait_for_code("expected-state").await });

        send_callback("/unexpected?code=ignored&state=expected-state").await;
        send_callback("/oauth/callback?code=expected-code&state=expected-state").await;

        assert_eq!(wait_for_code.await.unwrap().unwrap(), "expected-code");
    }
}
