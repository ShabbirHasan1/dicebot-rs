use ed25519_dalek::{PublicKey, Signature, Verifier};
use rand::Rng;
use regex::Regex;
use std::convert::TryInto;
use twilight_embed_builder::EmbedBuilder;
use twilight_model::application::callback::{CallbackData, InteractionResponse};
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::{ApplicationCommand, Interaction};
use twilight_model::channel::message::MessageFlags;
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();
    Router::new(())
        .post_async("/interactions", |mut req, ctx| async move {
            let public_key = PublicKey::from_bytes(
                hex::decode(
                    ctx.secret("PUBLIC_KEY")
                        .expect("Missing public key")
                        .to_string(),
                )
                .expect("Invalid public key")
                .as_slice(),
            )
            .expect("Invalid public key");
            let signature = Signature::new(
                hex::decode(
                    req.headers()
                        .get("X-Signature-Ed25519")
                        .expect("Missing signature")
                        .unwrap(),
                )
                .expect("Invalid signature")
                .as_slice()
                .try_into()
                .unwrap(),
            );
            let timestamp = req
                .headers()
                .get("X-Signature-Timestamp")
                .expect("Missing timestamp")
                .unwrap();
            let body = req.bytes().await.expect("Missing request body");
            match public_key.verify(
                &[timestamp.as_bytes(), body.as_slice()].concat(),
                &signature,
            ) {
                Ok(_) => {
                    let interaction: Interaction =
                        serde_json::from_slice(body.as_slice()).expect("Invalid request body");
                    match interaction {
                        Interaction::Ping(_) => {
                            return Response::from_json(&InteractionResponse::Pong);
                        }
                        Interaction::ApplicationCommand(command) => {
                            return handle_command(command);
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    return Response::error("Invalid request signature", 401);
                }
            }
            Response::error("Bad Request", 400)
        })
        .run(req, env)
        .await
}

fn handle_command(command: Box<ApplicationCommand>) -> Result<Response> {
    if command.data.name == "roll" {
        let mut die = 0;
        let mut count = 1;
        let mut modifier = 0;
        let mut gm = false;
        for option in command.data.options {
            match option {
                CommandDataOption::String { name, value } => {
                    if name == "dice" {
                        let re = Regex::new(r"^(\d*)[dD](\d+)$").unwrap();
                        if let Some(caps) = re.captures(value.as_str()) {
                            let c = caps.get(1).unwrap().as_str();
                            if !c.is_empty() {
                                count = c.parse().unwrap();
                                if count < 1 {
                                    return build_response(
                                        String::from("You can't roll less than one die!"),
                                        true,
                                    );
                                }
                                if count > 8 {
                                    return build_response(
                                        String::from("You can't roll more than eight dice!"),
                                        true,
                                    );
                                }
                            }
                            die = caps.get(2).unwrap().as_str().parse().unwrap();
                            if die < 4 {
                                return build_response(
                                    String::from("Your dice can't have less than four faces!"),
                                    true,
                                );
                            }
                            if die > 120 {
                                return build_response(
                                    String::from("Your dice can't have more than 120 faces!"),
                                    true,
                                );
                            }
                        } else {
                            return build_response(String::from("Please enter the dice you want to roll, e. g. `1d20` or `4d8`!"), true);
                        }
                    }
                }
                CommandDataOption::Integer { name, value } => {
                    if name == "modifier" {
                        if value <= 0 {
                            return build_response(
                                String::from("Your modifier can't be less than one!"),
                                true,
                            );
                        }
                        modifier = value
                    }
                }
                CommandDataOption::Boolean { name, value } => {
                    if name == "gm" {
                        gm = value
                    }
                }
                CommandDataOption::SubCommand { .. } => {}
            }
        }
        let mut rng = rand::thread_rng();
        return if count == 1 {
            let result = rng.gen_range(1..die + 1);
            if modifier != 0 {
                build_response(
                    format!(
                        "Your result is **{} *+ {}* = {}**",
                        result,
                        modifier,
                        result + modifier
                    ),
                    gm,
                )
            } else {
                build_response(format!("Your result is **{}**", result), gm)
            }
        } else {
            let mut results: Vec<String> = Vec::new();
            let mut result = 0;
            for _ in 0..count {
                let throw = rng.gen_range(1..die + 1);
                results.push(throw.to_string());
                result += throw;
            }
            if modifier != 0 {
                build_response(
                    format!(
                        "Your results are **({}) *+ {}* = {}**",
                        results.join(" + "),
                        modifier,
                        result + modifier
                    ),
                    gm,
                )
            } else {
                build_response(
                    format!(
                        "Your results are **({}) = {}**",
                        results.join(" + "),
                        result
                    ),
                    gm,
                )
            }
        };
    }
    Response::error("Bad Request", 400)
}

fn build_response(content: String, ephemeral: bool) -> Result<Response> {
    let mut flags = None;
    if ephemeral {
        flags = Some(MessageFlags::EPHEMERAL);
    }
    Response::from_json(&InteractionResponse::ChannelMessageWithSource(
        CallbackData {
            allowed_mentions: None,
            components: None,
            content: None,
            embeds: vec![EmbedBuilder::new()
                .description(content)
                .color(0xdd2e44)
                .build()
                .expect("Invalid embed")],
            flags,
            tts: None,
        },
    ))
}
