use serde::{Serialize, Deserialize};
use chess_network_protocol::*;

use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};

use crate::server;

pub enum ClientToGame {
    Move {
        board: [[Piece; 8]; 8],
        moves: Vec<Move>,
        joever: Joever,
        move_made: Move,
        turn: Color,
    },
    Handshake {
        board: [[Piece; 8]; 8],
        moves: Vec<Move>,
        features: Vec<Features>,
        turn: Color,
    },
}

pub struct GameToClient {
    pub move_made: Move,
}

trait Switch {
    fn switch_turn(&mut self);
}


fn switch_turn(turn: Color) -> Color {
    match turn {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

pub fn run(sender: Sender<ClientToGame>, receiver: Receiver<GameToClient>, server_color: Color) {
    let stream = TcpStream::connect("127.0.0.1:5000").unwrap();
    let mut de = serde_json::Deserializer::from_reader(&stream);

    let handshake = ClientToServerHandshake {
        server_color: server_color,
    };

    //send
    serde_json::to_writer(&stream, &handshake).unwrap();

    //receive
    let deserialized = ServerToClientHandshake::deserialize(&mut de).unwrap();
    println!("Recieved: {:?}", deserialized);

    let mut turn = switch_turn(server_color);

    sender.send(ClientToGame::Handshake {
        board: deserialized.board,
        moves: deserialized.moves,
        features: deserialized.features,
        turn: turn,
    }).unwrap();

    if turn == Color::White {
        let move_made = receiver.recv().unwrap();

        let mv = ClientToServer::Move(move_made.move_made);

        //send
        serde_json::to_writer(&stream, &mv).unwrap();
    }

    loop {
        let deserialized = ServerToClient::deserialize(&mut de).unwrap();
        println!("Recieved: {:?}", deserialized);

        match deserialized {
            ServerToClient::State { board, moves, joever, move_made } => {
                sender.send(ClientToGame::Move { 
                    board, 
                    moves, 
                    joever, 
                    move_made: move_made, 
                    turn,
                }).unwrap();
            },
            ServerToClient::MoveRequested => {
                let move_made = receiver.recv().unwrap();

                //send
                serde_json::to_writer(&stream, &move_made).unwrap();
            },
            ServerToClient::GameEnded => {
                break;
            },
        }

        let move_made = receiver.recv().unwrap();

        //send
        serde_json::to_writer(&stream, &move_made).unwrap();
    }

    //assumes that the client is white
    let moved = ClientToServer::Move(Move { 
        start_x: 0, 
        start_y: 0, 
        end_x: 1, 
        end_y: 1, 
        promotion: Piece::None, 
    });

    //send
    serde_json::to_writer(&stream, &moved).unwrap();

    //receive
    let deserialized = ServerToClient::deserialize(&mut de).unwrap();
    println!("Recieved: {:?}", deserialized);

    //receive
    let deserialized = ServerToClient::deserialize(&mut de).unwrap();
    println!("Recieved: {:?}", deserialized);
}