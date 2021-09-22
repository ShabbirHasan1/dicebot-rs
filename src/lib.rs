use crate::roll::Roll;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use std::convert::TryInto;
use twilight_embed_builder::EmbedBuilder;
use twilight_model::application::callback::{CallbackData, InteractionResponse};
use twilight_model::application::component::button::ButtonStyle;
use twilight_model::application::component::{ActionRow, Component};
use twilight_model::application::component::{Button, ComponentType};
use twilight_model::application::interaction::{
    ApplicationCommand, Interaction, MessageComponentInteraction,
};
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::ReactionType;
use worker::*;

mod roll;
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
                        Interaction::MessageComponent(component) => {
                            match component.data.component_type {
                                ComponentType::Button => {
                                    return handle_button(component);
                                }
                                _ => {}
                            }
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

fn handle_button(button: Box<MessageComponentInteraction>) -> Result<Response> {
    build_response_ok(Roll::from_custom_id(button.data.custom_id))
}

fn handle_command(command: Box<ApplicationCommand>) -> Result<Response> {
    if command.data.name == "roll" {
        let roll = Roll::from_command(command);
        return match roll {
            Ok(roll) => build_response_ok(roll),
            Err(err) => build_response_err(err),
        };
    }
    Response::error("Bad Request", 400)
}

fn build_response_err(err: String) -> Result<Response> {
    Response::from_json(&InteractionResponse::ChannelMessageWithSource(
        CallbackData {
            allowed_mentions: None,
            components: None,
            content: None,
            embeds: vec![EmbedBuilder::new()
                .description(err)
                .color(0xdd2e44)
                .build()
                .expect("Invalid embed")],
            flags: Some(MessageFlags::EPHEMERAL),
            tts: None,
        },
    ))
}

fn build_response_ok(roll: Roll) -> Result<Response> {
    Response::from_json(&InteractionResponse::ChannelMessageWithSource(
        CallbackData {
            allowed_mentions: None,
            components: Some(vec![Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: Some(roll.to_custom_id()),
                    disabled: false,
                    emoji: Some(ReactionType::Unicode {
                        name: "\u{1F3B2}".to_string(),
                    }),
                    label: Some("Reroll".to_string()),
                    style: ButtonStyle::Secondary,
                    url: None,
                })],
            })]),
            content: None,
            embeds: vec![EmbedBuilder::new()
                .description(roll.to_string())
                .color(0xdd2e44)
                .build()
                .unwrap()],
            flags: if roll.ephemeral() {
                Some(MessageFlags::EPHEMERAL)
            } else {
                None
            },
            tts: None,
        },
    ))
}
