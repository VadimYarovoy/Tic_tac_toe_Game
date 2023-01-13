use std::sync::Arc;

use image::{ImageBuffer, Rgba, Rgb};
use serenity::all::{UserId, CommandInteraction, Message};
use serenity::all::CreateCommand;
use tokio::sync::Mutex;
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;

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

        let horizontal_scratch = image::open("./resources/1.png").expect("1.png").into_rgba8();
        let vertical_scratch = image::open("./resources/2.png").expect("2.png").into_rgba8();
        let diagonal_scratch_1 = image::open("./resources/3.png").expect("3.png").into_rgba8();
        let diagonal_scratch_2 = image::open("./resources/4.png").expect("4.png").into_rgba8();

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
    draw_filled_rect_mut(
        &mut canvas,
        Rect::at(0, 0).of_size(300, 300),
        BACKGROUND,
    );

    draw_filled_rect_mut(
        &mut canvas,
        Rect::at(98, 0).of_size(4, 300),
        GRAY,
    );

    draw_filled_rect_mut(
        &mut canvas,
        Rect::at(198, 0).of_size(4, 300),
        GRAY,
    );

    draw_filled_rect_mut(
        &mut canvas,
        Rect::at(0, 98).of_size(300, 4),
        GRAY,
    );

    draw_filled_rect_mut(
        &mut canvas,
        Rect::at(0, 198).of_size(300, 4),
        GRAY,
    );

    canvas
}

impl Game {
	pub fn register_play() -> CreateCommand {
		CreateCommand::new("play")
			.description("Start the game")
	}
	
	pub fn register_stop() -> CreateCommand {
		CreateCommand::new("stop")
			.description("Unimplemented")
	}
}

pub async fn command(&self, ctx: Context, interaction: CommandInteraction) {
	if interaction.data.name == "stop" {
		interaction.create_response(&ctx.http, CreateInteractionResponse::Message(
			CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.content("Unimplemented!")
		))
		.await
		.unwrap();

		return;
	}

	if self.is_player_already_in_game(&ctx.http, &interaction).await {
		return;
	}
}

impl Game {
	async fn is_player_already_in_game(&self, http: &Http, interaction: &CommandInteraction) -> bool {
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
                    interaction.create_response(http, message)
                    .await
                    .unwrap();

                    return true;
                }
            }
        }

        let sessions = self.sessions.lock().await;

        for session in &*sessions {
            let session = session.lock().await;

            if session.player.0 == interaction.user.id
                || session.player2.0 == interaction.user.id
            {
                interaction.create_response(http, message)
                .await
                .unwrap();

                return true;
            }
        }

        false
    }

}