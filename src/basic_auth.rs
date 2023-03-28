//! Basic HTTP Authorization Schema module.
//!

use bcrypt::verify as bcrypt_verify;
use headers::{authorization::Basic, Authorization, HeaderMapExt};
use hyper::StatusCode;

/// Check for a `Basic` HTTP Authorization Schema of an incoming request
/// and uses `bcrypt` for password hashing verification.
pub fn check_request(
    headers: &http::HeaderMap,
    userid: &str,
    password: &str,
) -> Result<(), StatusCode> {
    if let Some(ref credentials) = headers.typed_get::<Authorization<Basic>>() {
        if credentials.0.username() == userid {
            return match bcrypt_verify(credentials.0.password(), password) {
                Ok(valid) => {
                    if valid {
                        Ok(())
                    } else {
                        Err(StatusCode::UNAUTHORIZED)
                    }
                }
                Err(err) => {
                    tracing::error!("bcrypt password verification error: {:?}", err);
                    Err(StatusCode::UNAUTHORIZED)
                }
            };
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use super::check_request;
    use headers::HeaderMap;

    #[test]
    fn test_valid_auth() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic anE6anE=".parse().unwrap());
        assert!(check_request(
            &headers,
            "jq",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_ok());
    }

    #[test]
    fn test_invalid_auth_header() {
        let headers = HeaderMap::new();
        assert!(check_request(&headers, "jq", "").is_err());
    }

    #[test]
    fn test_invalid_auth_pairs() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic anE6anE=".parse().unwrap());
        assert!(check_request(&headers, "xyz", "").is_err());
    }

    #[test]
    fn test_invalid_auth() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic anE6anE=".parse().unwrap());
        assert!(check_request(
            &headers,
            "abc",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_err());
        assert!(check_request(&headers, "jq", "password").is_err());
        assert!(check_request(&headers, "", "password").is_err());
        assert!(check_request(&headers, "jq", "").is_err());
    }

    #[test]
    fn test_invalid_auth_encoding() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic xyz".parse().unwrap());
        assert!(check_request(
            &headers,
            "jq",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_err());
    }

    #[test]
    fn test_invalid_auth_encoding2() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "abcd".parse().unwrap());
        assert!(check_request(
            &headers,
            "jq",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_err());
    }
}
