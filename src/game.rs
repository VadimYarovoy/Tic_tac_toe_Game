use std::sync::Arc;

use image::{ImageBuffer, Rgb, Rgba};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use serenity::all::CreateCommand;
use serenity::all::{CommandInteraction, Message, UserId};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct Game {
    x_image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    o_image: ImageBuffer<Rgb<u8>, Vec<u8>>,

    horizontal_scratch: ImageBuffer<Rgba<u8>, Vec<u8>>,
    vertical_scratch: ImageBuffer<Rgba<u8>, Vec<u8>>,
    diagonal_scratch_1: ImageBuffer<Rgba<u8>, Vec<u8>>, // Left to right
    diagonal_scratch_2: ImageBuffer<Rgba<u8>, Vec<u8>>, // Right to left

    new_game_canvas: ImageBuffer<Rgb<u8>, Vec<u8>>,

    wait_user: Mutex<Option<(UserId, CommandInteraction, String, Message)>>,

    sessions: Mutex<Vec<Arc<Mutex<GameSession>>>>,
}

#[derive(Clone, Copy, PartialEq)]
enum GameCell {
    None,
    First,
    Second,
}

impl Default for GameCell {
    fn default() -> Self {
        GameCell::None
    }
}

struct GameSession {
    player: (UserId, CommandInteraction, String, Message), // Third element is a name of player
    player2: (UserId, CommandInteraction, String, Option<Message>), // No message in a same channel

    stage: usize,
    cursor_pos: usize,

    map: [GameCell; 9],
    canvas: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl Game {
    pub fn new() -> Self {
        let x_image = image::open("./resources/x.png").expect("x.png").into_rgb8();
        let o_image = image::open("./resources/o.png").expect("o.png").into_rgb8();

        let horizontal_scratch = image::open("./resources/1.png")
            .expect("1.png")
            .into_rgba8();
        let vertical_scratch = image::open("./resources/2.png")
            .expect("2.png")
            .into_rgba8();
        let diagonal_scratch_1 = image::open("./resources/3.png")
            .expect("3.png")
            .into_rgba8();
        let diagonal_scratch_2 = image::open("./resources/4.png")
            .expect("4.png")
            .into_rgba8();

        let new_game_canvas = draw_new_game_canvas();

        Self {
            x_image,
            o_image,

            horizontal_scratch,
            vertical_scratch,
            diagonal_scratch_1,
            diagonal_scratch_2,

            new_game_canvas,

            ..Default::default()
        }
    }
}

fn draw_new_game_canvas() -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut canvas = ImageBuffer::new(300, 300);

    // Background
    draw_filled_rect_mut(&mut canvas, Rect::at(0, 0).of_size(300, 300), BACKGROUND);

    draw_filled_rect_mut(&mut canvas, Rect::at(98, 0).of_size(4, 300), GRAY);

    draw_filled_rect_mut(&mut canvas, Rect::at(198, 0).of_size(4, 300), GRAY);

    draw_filled_rect_mut(&mut canvas, Rect::at(0, 98).of_size(300, 4), GRAY);

    draw_filled_rect_mut(&mut canvas, Rect::at(0, 198).of_size(300, 4), GRAY);

    canvas
}

impl Game {
    pub fn register_play() -> CreateCommand {
        CreateCommand::new("play").description("Start the game")
    }

    pub fn register_stop() -> CreateCommand {
        CreateCommand::new("stop").description("Unimplemented")
    }
}

pub async fn command(&self, ctx: Context, interaction: CommandInteraction) {
    if interaction.data.name == "stop" {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .content("Unimplemented!"),
                ),
            )
            .await
            .unwrap();

        return;
    }

    if self
        .is_player_already_in_game(&ctx.http, &interaction)
        .await
    {
        return;
    }
}

impl Game {
    async fn is_player_already_in_game(
        &self,
        http: &Http,
        interaction: &CommandInteraction,
    ) -> bool {
        let message = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .embed(
                    CreateEmbed::new()
                        .title("Start a new game")
                        .description("You have already in the game. For starting a new game you should use the `/stop` command.")
                )
        );

        {
            if let Some(val) = self.wait_user.lock().await.as_ref() {
                if val.0 == interaction.user.id {
                    interaction.create_response(http, message).await.unwrap();

                    return true;
                }
            }
        }

        let sessions = self.sessions.lock().await;

        for session in &*sessions {
            let session = session.lock().await;

            if session.player.0 == interaction.user.id || session.player2.0 == interaction.user.id {
                interaction.create_response(http, message).await.unwrap();

                return true;
            }
        }

        false
    }
}

pub async fn command(&self, ctx: Context, interaction: CommandInteraction) {
    let (player, player2) = {
        let val = { self.wait_user.lock().await.take() };

        let name = match &interaction.member {
            Some(val) => val
                .nick
                .clone()
                .unwrap_or_else(|| interaction.user.name.clone()),
            None => interaction.user.name.clone(),
        };

        if let Some(val) = val {
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .embed(CreateEmbed::new().title("Please, wait")),
                    ),
                )
                .await
                .unwrap();

            if interaction.channel_id != val.1.channel_id {
                let message = interaction
                    .channel_id
                    .send_message(
                        &ctx.http,
                        CreateMessage::new().embed(CreateEmbed::new().title(format!(
                            "The game between {} and {} in progress!",
                            val.2, name,
                        ))),
                    )
                    .await
                    .unwrap();

                (val, (interaction.user.id, interaction, name, Some(message)))
            } else {
                (val, (interaction.user.id, interaction, name, None))
            }
        } else {
            let icon_url = interaction
                .user
                .avatar_url()
                .unwrap_or_else(|| interaction.user.default_avatar_url());

            let message = interaction
                .channel_id
                .send_message(
                    &ctx.http,
                    CreateMessage::new().embed(
                        CreateEmbed::new()
                            .author(CreateEmbedAuthor::new(name.clone()).icon_url(icon_url))
                            .title(format!("{} wants to play tic-tac-toe game!", name))
                            .description(
                                "You can join to him/her/them by using the `/play` command.",
                            ),
                    ),
                )
                .await
                .unwrap();

            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .embed(CreateEmbed::new().title("Please, wait for second player...")),
                    ),
                )
                .await
                .unwrap();

            *self.wait_user.lock().await = Some((interaction.user.id, interaction, name, message));
            return;
        }
    };
}

impl Game {
	async fn process_session(&self, http: &Http, session: &mut GameSession) {
        match session.stage {
            0 => {
                show_game_message(
                    http,
                    &session.player.1,
                    session.cursor_pos,
                    &session.map,
                    &session.canvas,
                ).await;

                show_wait_and_common_message(
                    http,
                    &session.player2.1,
                    &session.canvas,
                    &session.player.2,
                    &session.player2.2,
                    &mut session.player.3,
                    session.player2.3.as_mut(),
                ).await;
            }
            1 => {
                show_game_message(
                    http,
                    &session.player2.1,
                    session.cursor_pos,
                    &session.map,
                    &session.canvas,
                ).await;

                show_wait_and_common_message(
                    http,
                    &session.player.1,
                    &session.canvas,
                    &session.player.2,
                    &session.player2.2,
                    &mut session.player.3,
                    session.player2.3.as_mut(),
                ).await;
            }
            _ => unreachable!(),
        }
    }
}


use serenity::all::{ComponentInteraction, EditInteractionResponse, EditMessage};

async fn show_wait_and_common_message(
    http: &Http,
    interaction: &CommandInteraction,
    canvas: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    player_name: &str,
    player2_name: &str,
    common_message: &mut Message,
    common_message2: Option<&mut Message>,
) {
    let embed = CreateEmbed::new()
        .title("Game in process")
        .description("Waiting for your turn.")
        .thumbnail("attachment://thumbnail.png");

    let action_row = generate_disabled_action_row();
    let attachment = generate_attachment_rgb8(canvas, "canvas.png");

    interaction.edit_response(http, EditInteractionResponse::new()
        .add_embed(embed)
        .components(vec![action_row])
        .new_attachment(attachment.clone())
    ).await.unwrap();

    let edited_message = EditMessage::new()
        .embed(CreateEmbed::new()
            .title(format!(
                "Game between {} and {} in the progress!",
                player_name,
                player2_name,
            ))
            .description("You can play this game too by using the `/play` command.")
            .attachment("canvas.png")
        )
        .attachment(attachment);

    if let Some(val) = common_message2 {
        val.edit(http, edited_message.clone()).await.unwrap();
    }

    common_message.edit(http, edited_message).await.unwrap();
}

async fn show_game_message(
    http: &Http,
    interaction: &CommandInteraction,
    cursor_pos: usize,
    map: &[GameCell],
    canvas: &ImageBuffer<Rgb<u8>, Vec<u8>>,
) {
    let embed = CreateEmbed::new()
    .title("Your turn")
    .description("Press arrows buttons for moving selection square.");

    let action_row = if map[cursor_pos] != GameCell::None {
        generate_game_action_row(true, cursor_pos)
    }
    else {
        generate_game_action_row(false, cursor_pos)
    };

    let mut cloned = canvas.clone();

    draw_select_outline(&mut cloned, cursor_pos);

    interaction.edit_response(http, EditInteractionResponse::new()
        .embed(embed)
        .components(vec![action_row])
        .new_attachment(generate_attachment_rgb8(&cloned, "canvas.png"))
    )
    .await
    .unwrap();
}

async fn update_game_message(http: &Http, interaction: &ComponentInteraction, session: &GameSession) {
    let embed = CreateEmbed::new()
        .title("Your turn")
        .description("Press arrows buttons for moving selection square.");

    let action_row = if session.map[session.cursor_pos] != GameCell::None {
        generate_game_action_row(true, session.cursor_pos)
    }
    else {
        generate_game_action_row(false, session.cursor_pos)
    };

    let mut cloned = session.canvas.clone();

    draw_select_outline(&mut cloned, session.cursor_pos);

    interaction.edit_response(http, EditInteractionResponse::new()
        .embed(embed)
        .components(vec![action_row])
        .new_attachment(generate_attachment_rgb8(&cloned, "canvas.png"))
    )
    .await
    .unwrap();
}