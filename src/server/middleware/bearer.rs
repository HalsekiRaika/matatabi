use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use actix_web::dev::ServiceRequest;
use actix_web::http::header::{HeaderName, HeaderValue, InvalidHeaderValue, TryIntoHeaderValue};
use actix_web::http::StatusCode;
use actix_web::{HttpMessage, HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::web::{BufMut, BytesMut};
use futures::future::{Ready, ready};

const BEARER_SC: &str = "Bearer";
const SCHEME_NAME_NUM: usize = 8;

#[derive(Clone)]
pub struct Credentials(Bearer);

impl Credentials {
    pub fn get_token(&self) -> &str {
        self.0.get_token().as_ref()
    }

    pub fn from_req(req: &ServiceRequest) -> Ready<Result<Self, AuthError>> {
        use actix_web::http::header::Header;

        ready(BearerHeader::parse(req)
            .map(|a| Credentials(a.into_value()))
            .map_err(|_| {
                AuthError::realm_new("cannot parse bearer token.")
            }),
        )
    }
}

#[derive(Debug)]
pub struct AuthError { reason: String, status_code: StatusCode }

impl AuthError {
    pub fn realm_new(reason: impl Into<String>) -> Self {
        Self {
            reason: format!("realm=\"{}\"", reason.into()),
            status_code: StatusCode::UNAUTHORIZED
        }
    }

    pub fn error_new(reason: impl Into<String>) -> Self {
        Self {
            reason: format!("error=\"{}\"", reason.into()),
            status_code: StatusCode::UNAUTHORIZED
        }
    }

    pub fn new(reason: impl Into<String>, status_code: StatusCode) -> Self {
        Self { reason: reason.into(), status_code }
    }
}

impl std::error::Error for AuthError { }
impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.status_code, f)
    }
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code)
            .insert_header(("WWW-Authenticate", format!("Bearer {}", self.reason.clone())))
            .finish()
    }
}

// Cow<'static, str>
// -- referenced by https://users.rust-lang.org/t/idiomatic-string-parmeter-types-str-vs-asref-str-vs-into-string/7934
// and this source code custom for this proj from
// -- https://github.com/actix/actix-extras/blob/master/actix-web-httpauth/src/extractors/bearer.rs
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bearer {
    token: Cow<'static, str>
}

impl Bearer {
    #[allow(unused)]
    pub fn new(raw: impl Into<Cow<'static, str>>) -> Self {
        Self { token: raw.into() }
    }

    pub fn get_token(&self) -> &Cow<'static, str> {
        &self.token
    }

    pub fn parse(header: &HeaderValue) -> Result<Self, ParseError> {
        if header.len() < SCHEME_NAME_NUM {
            return Err(ParseError::Invalid);
        }

        let mut parts = header.to_str().unwrap().splitn(2, ' ');
        match parts.next() {
            Some(token) if token == BEARER_SC => (),
            _ => return Err(ParseError::MissingTokenSchemeName)
        }

        let token = parts.next().ok_or(ParseError::Invalid)?;

        Ok(Self { token: token.to_string().into() })
    }
}

impl TryIntoHeaderValue for Bearer {
    type Error = InvalidHeaderValue;

    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        let mut mut_cap = BytesMut::with_capacity(7 + self.get_token().len());
        mut_cap.put(&b"Bearer "[..]);
        mut_cap.extend_from_slice(self.get_token().as_bytes());

        HeaderValue::from_maybe_shared(mut_cap.freeze())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BearerHeader(Bearer);

impl BearerHeader {
    pub fn into_value(self) -> Bearer {
        self.0
    }
}

impl actix_web::http::header::Header for BearerHeader {
    fn name() -> HeaderName {
        actix_web::http::header::AUTHORIZATION
    }

    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, actix_web::error::ParseError> {
        let header = msg.headers().get(actix_web::http::header::AUTHORIZATION)
                .ok_or(actix_web::error::ParseError::Header)?;
        let from_header = Bearer::parse(header)
                .map_err(|_| actix_web::error::ParseError::Header)?;
        Ok(BearerHeader(from_header))
    }
}

impl TryIntoHeaderValue for BearerHeader {
    type Error = <Bearer as TryIntoHeaderValue>::Error;
    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        self.0.try_into_value()
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum ParseError {
    Invalid,
    MissingTokenSchemeName,
    MissingTokenString(&'static str),
    FailedToStr(actix_web::http::header::ToStrError),
}

impl Display for ParseError {
    fn fmt(&self, format: &mut Formatter<'_>) -> std::fmt::Result {
        format.write_str(&*match self {
            ParseError::Invalid => "Invalid header value.".to_string(),
            ParseError::MissingTokenSchemeName => "Missing authenticate scheme.".to_string(),
            ParseError::MissingTokenString(_) => "Missing header value.".to_string(),
            ParseError::FailedToStr(err) => err.to_string(),
        })
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::Invalid => None,
            ParseError::MissingTokenSchemeName => None,
            ParseError::MissingTokenString(_) => None,
            ParseError::FailedToStr(err) => Some(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        // This is going to work, right?
        let value = HeaderValue::from_static("Bearer mF_9.B5f-4.1JqM");
        let token = Bearer::parse(&value);

        assert!(token.is_ok());
        let token = token.unwrap();
        assert_eq!(token.get_token(), "mF_9.B5f-4.1JqM");
    }

    #[test]
    fn test_parse_empty_pattern() {
        // silence
        let value = HeaderValue::from_static("");
        let token = Bearer::parse(&value);

        assert!(token.is_err());
    }

    #[test]
    fn test_parse_wrong_token_pattern() {
        // Oops. I guess I shouted too much and didn't get the requirements across.
        let value = HeaderValue::from_static("YEAHHHHHHHHHHHHHHHHHH");
        let token = Bearer::parse(&value);

        assert!(token.is_err());
    }

    #[test]
    fn test_parse_token_string_empty_pattern() {
        // scheme name only
        let value = HeaderValue::from_static("Bearer ");
        let token = Bearer::parse(&value);

        assert!(token.is_err());
    }
}