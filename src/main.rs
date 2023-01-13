mod game;
mod ping;

use serenity::all::Interaction;
use serenity::async_trait;
use serenity::all::Ready;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::prelude::*;

use game::Game;

struct Handler {
    game: Game,
}

impl Handler {
    fn new() -> Self {
        Self {
            game: Game::new(),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                match command.data.name.as_str() {
                    "ping" => ping::command(ctx, command).await,
                    "play" => self.game.command(ctx, command).await,
                    _ => {
                        command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .ephemeral(true)
                                .content("Invalid command!")
                        ))
                        .await
                        .expect("failed to create response");
                    }
                }               
            }

            Interaction::Component(component) => {
                self.game.component(ctx, component).await;
            }

            _ => (),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} has connected!", ready.user.name);

        let guild = ready.guilds[0];
        assert_eq!(guild.unavailable, true);
        let guild_id = guild.id;

        guild_id.set_application_commands(&ctx.http, vec![
            Game::register_play(),
            Game::register_stop(),
            ping::register(), 
        ])
        .await
        .expect("failed to create application command");
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(include_str!("./../token.txt"), intents)
        .event_handler(Handler::new())
        .await
        .expect("Failed to create client!");

    if let Err(err) = client.start().await {
        eprintln!("Client error: {err:?}");
    }
}