use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct State {
    board: [[Option<Piece>; 8]; 8],
    moves: Vec<Move>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Piece {
    piece: Role,
    color: Color,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum Role {
    Pawn,
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum Color {
    White,
    Black,
}

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    start_x: u8,
    start_y: u8,
    end_x: u8,
    end_y: u8,
    promotion: Option<Role>,
}

use std::io::prelude::*;
use std::net::TcpListener;

pub fn run() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;

    // accept connections and process them serially
    let (stream, _addr) = listener.accept()?;

    let mut de = serde_json::Deserializer::from_reader(stream);
    let deserialized = State::deserialize(&mut de)?;

    println!("Recieved: {:?}", deserialized);

    Ok(())
}