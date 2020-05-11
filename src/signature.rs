use ring::{digest, hmac};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use std::env::var;

#[derive(Debug)]
pub enum GithubSignatureError {
    Missing,
    UnexpectedHeaders,
    Invalid,
}

pub struct GithubSignature(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for GithubSignature {
    type Error = GithubSignatureError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let signature_secret = var("GITHUB_SECRET");

        match signature_secret {
            Ok(_) => {
                let keys: Vec<_> = request.headers().get("X-Hub-Signature").collect();
                match keys.len() {
                    0 => Outcome::Failure((Status::Unauthorized, GithubSignatureError::Missing)),
                    1 => Outcome::Success(GithubSignature(keys[0].to_string())),
                    _ => Outcome::Failure((
                        Status::BadRequest,
                        GithubSignatureError::UnexpectedHeaders,
                    )),
                }
            }
            Err(_) => Outcome::Success(GithubSignature("".to_owned())),
        }
    }
}

pub fn verify_signature(secret: String, signature: &str, raw_body: &str) -> bool {
    let signing_key = hmac::SigningKey::new(&digest::SHA1, &secret.as_bytes());
    let generated_signature = hmac::sign(&signing_key, raw_body.as_bytes());
    let signature_hex = generated_signature
        .as_ref()
        .iter()
        .fold("".to_owned(), |acc, x| {
            let hex_char = &format!("{:x?}", x);
            let escaped_hex = if hex_char.len() == 1 {
                "0".to_owned() + hex_char
            } else {
                hex_char.to_owned()
            };
            acc + &escaped_hex
        });

    signature_hex == signature.replace("sha1=", "")
}
