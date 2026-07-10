//! Shared-password access control for the private Flight Review pilot.

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

const COOKIE_NAME: &str = "flight_review_access";
const SESSION_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Clone)]
pub struct AccessControl {
    password_hash: [u8; 32],
    session_key: [u8; 32],
    secure_cookie: bool,
}

impl AccessControl {
    pub fn from_env() -> Result<Option<Self>, String> {
        let Some(password) = std::env::var("ACCESS_PASSWORD")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        if password.len() < 16 {
            return Err("ACCESS_PASSWORD must be at least 16 characters".into());
        }

        let secure_cookie = std::env::var("ACCESS_COOKIE_SECURE")
            .ok()
            .map(|value| value != "false" && value != "0")
            .unwrap_or_else(|| std::env::var_os("RAILWAY_ENVIRONMENT_ID").is_some());
        Ok(Some(Self::new(&password, secure_cookie)))
    }

    pub fn new(password: &str, secure_cookie: bool) -> Self {
        Self {
            password_hash: digest_with_context(b"flight-review-password-v1", password),
            session_key: digest_with_context(b"flight-review-session-v1", password),
            secure_cookie,
        }
    }

    fn verify_password(&self, candidate: &str) -> bool {
        let candidate_hash = digest_with_context(b"flight-review-password-v1", candidate);
        bool::from(self.password_hash.ct_eq(&candidate_hash))
    }

    fn issue_cookie(&self) -> String {
        let expires = unix_seconds().saturating_add(SESSION_SECONDS);
        let payload = expires.to_string();
        let signature = self.sign(&payload);
        let secure = if self.secure_cookie { "; Secure" } else { "" };
        format!(
            "{COOKIE_NAME}={payload}.{signature}; Path=/; Max-Age={SESSION_SECONDS}; HttpOnly; SameSite=Strict{secure}"
        )
    }

    fn clear_cookie(&self) -> String {
        let secure = if self.secure_cookie { "; Secure" } else { "" };
        format!("{COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict{secure}")
    }

    fn is_authenticated(&self, headers: &HeaderMap) -> bool {
        let Some(value) = cookie_value(headers, COOKIE_NAME) else {
            return false;
        };
        let Some((expires, signature)) = value.split_once('.') else {
            return false;
        };
        let Ok(expires_at) = expires.parse::<u64>() else {
            return false;
        };
        if expires_at < unix_seconds() {
            return false;
        }
        let Ok(signature) = decode_hex(signature) else {
            return false;
        };
        let Ok(mac) = HmacSha256::new_from_slice(&self.session_key) else {
            return false;
        };
        let mut mac = mac;
        mac.update(expires.as_bytes());
        mac.verify_slice(&signature).is_ok()
    }

    fn sign(&self, payload: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.session_key)
            .expect("SHA-256 accepts keys of any size");
        mac.update(payload.as_bytes());
        encode_hex(&mac.finalize().into_bytes())
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    password: String,
}

#[derive(Serialize)]
pub struct SessionResponse {
    authenticated: bool,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

pub async fn login(
    State(state): State<Arc<crate::AppState>>,
    Json(request): Json<LoginRequest>,
) -> Response {
    let Some(access) = &state.access_control else {
        return Json(SessionResponse {
            authenticated: true,
        })
        .into_response();
    };
    if !access.verify_password(&request.password) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Access code not recognized",
            }),
        )
            .into_response();
    }

    let mut response = Json(SessionResponse {
        authenticated: true,
    })
    .into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&access.issue_cookie()).expect("valid access cookie"),
    );
    response
}

pub async fn logout(State(state): State<Arc<crate::AppState>>) -> Response {
    let mut response = Json(SessionResponse {
        authenticated: false,
    })
    .into_response();
    if let Some(access) = &state.access_control {
        response.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&access.clear_cookie()).expect("valid access cookie"),
        );
    }
    response
}

pub async fn session(
    State(state): State<Arc<crate::AppState>>,
    headers: HeaderMap,
) -> Json<SessionResponse> {
    let authenticated = state
        .access_control
        .as_ref()
        .is_none_or(|access| access.is_authenticated(&headers));
    Json(SessionResponse { authenticated })
}

pub async fn require_access(
    State(state): State<Arc<crate::AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let authenticated = state
        .access_control
        .as_ref()
        .is_none_or(|access| access.is_authenticated(request.headers()));
    if authenticated {
        next.run(request).await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .find_map(|cookie| cookie.strip_prefix(&format!("{name}=")))
}

fn digest_with_context(context: &[u8], value: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(context);
    digest.update(value.as_bytes());
    digest.finalize().into()
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_hex(value: &str) -> Result<Vec<u8>, ()> {
    if !value.len().is_multiple_of(2) {
        return Err(());
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).ok_or(())?;
            let low = (pair[1] as char).to_digit(16).ok_or(())?;
            Ok(((high << 4) | low) as u8)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_password_and_rejects_other_values() {
        let access = AccessControl::new("test-password-with-enough-entropy", false);
        assert!(access.verify_password("test-password-with-enough-entropy"));
        assert!(!access.verify_password("different-password-with-entropy"));
    }

    #[test]
    fn signed_cookie_authenticates_and_tampering_does_not() {
        let access = AccessControl::new("test-password-with-enough-entropy", false);
        let cookie = access.issue_cookie();
        let pair = cookie.split(';').next().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_str(pair).unwrap());
        assert!(access.is_authenticated(&headers));

        let tampered = pair.replace("flight_review_access=", "flight_review_access=9");
        headers.insert(header::COOKIE, HeaderValue::from_str(&tampered).unwrap());
        assert!(!access.is_authenticated(&headers));
    }
}
