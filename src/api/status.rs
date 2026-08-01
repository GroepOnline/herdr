use std::io;
use std::path::Path;
use std::time::Duration;

use crate::api::schema::{Method, Request, ResponseResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub version: Option<String>,
    pub protocol: Option<u32>,
    pub capabilities: Option<crate::api::schema::ServerCapabilities>,
}

pub fn read_runtime_status_at(
    socket_path: &Path,
    timeout: Duration,
) -> io::Result<Option<RuntimeStatus>> {
    if !socket_path.exists() {
        return Ok(None);
    }

    let client = crate::api::client::ApiClient::for_target(
        crate::api::client::ConnectionTarget::SocketPath(socket_path.to_path_buf()),
    );
    let request = Request {
        id: "runtime:status".into(),
        method: Method::Ping(crate::api::schema::PingParams::default()),
    };
    let response = client
        .request_value_with_timeout(&request, timeout)
        .and_then(crate::api::client::parse_response_value);
    let response = match response {
        Ok(response) => response,
        Err(crate::api::client::ApiClientError::Io(err))
            if matches!(
                err.kind(),
                io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::NotFound
                    | io::ErrorKind::TimedOut
            ) =>
        {
            return Ok(None);
        }
        Err(err) => return Err(io::Error::other(err)),
    };
    match response.result {
        ResponseResult::Pong {
            version,
            protocol,
            capabilities,
        } => Ok(Some(RuntimeStatus {
            version: Some(version),
            protocol: Some(protocol),
            capabilities,
        })),
        result => Err(io::Error::other(format!(
            "server status request returned unexpected result: {result:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_read_runtime_status_at_missing_file() {
        // Generate a unique path instead of reusing a shared one, so the test
        // never unlinks a socket owned by a concurrent run or another process.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let socket_path = std::env::temp_dir().join(format!(
            "herdr-missing-runtime-status-{}-{nanos}.sock",
            std::process::id()
        ));
        assert!(!socket_path.exists());

        let result = read_runtime_status_at(&socket_path, Duration::from_millis(100));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
