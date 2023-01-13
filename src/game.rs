use std::sync::Arc;

use image::{ImageBuffer, Rgba, Rgb};
use serenity::all::{UserId, CommandInteraction, Message};
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