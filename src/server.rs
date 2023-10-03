use serde::{Serialize, Deserialize};
use chess_network_protocol::*;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Sender, Receiver};

use redkar_chess::Game;

pub const FEATURES: Vec<Features> = vec![];

use crate::redkar_chess_utils::*;

pub enum ServerToGame {
    Handshake {
        game: Game,
        server_color: Color,
    },
    State {
        game: Game,
        turn: Color,
        move_made: Move,
        joever: Joever,
    },
    Error {
        error: redkar_chess::MoveError,
    },
}

pub struct GameToServer {
    pub move_made: Move,
}

fn switch_turn(turn: Color) -> Color {
    match turn {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

pub fn run(sender: Sender<ServerToGame>, receiver: Receiver<GameToServer>) {
    let listener = TcpListener::bind("127.0.0.1:5000").unwrap();

    // accept connections and process them serially
    let (stream, _addr) = listener.accept().unwrap();
    let mut de = serde_json::Deserializer::from_reader(&stream);

    //receive
    let deserialized = ClientToServerHandshake::deserialize(&mut de).unwrap();
    println!("Received: {:?}", deserialized);

    let mut game = Game::new_game();

    sender.send(ServerToGame::Handshake {
        game: game,
        server_color: deserialized.server_color,
    }).unwrap();

    let handshake = ServerToClientHandshake {
        features: FEATURES,
        board: game.board.into_network(),
        moves: vec![],
        joever: Joever::Ongoing,
    };

    //send
    serde_json::to_writer(&stream, &handshake).unwrap();

    let mut turn = deserialized.server_color.clone();

    if turn == Color::White {
        turn = make_move(sender, &receiver, turn, &stream, &mut game)
    }

    loop {
        //receive
        let deserialized = ClientToServer::deserialize(&mut de).unwrap();
        println!("Received: {:?}", deserialized);

        match deserialized {
            ClientToServer::Move(move_made) => {
                match game.do_move(move_made.into_chess()) {
                    Ok(d) => {
                        sender.send(ServerToGame::State {
                            game: game.clone(),
                            turn: turn.clone(),
                            move_made: move_made,
                            joever: d.into_network(),
                        }).unwrap();

                        let state = ServerToClient::State {
                            board: game.board.into_network(),
                            moves: vec![],
                            joever: d.into_network(),
                            move_made: move_made,
                        };

                        //send
                        serde_json::to_writer(stream, &state).unwrap();

                        turn = switch_turn(turn)
                    }
                    Err(e) => {
                        sender.send(ServerToGame::Error {
                            error: e,
                        }).unwrap();
                    }
                }
            },
            ClientToServer::Resign => { panic!("Resign not implemented") }
            ClientToServer::Draw => { panic!("Draw not implemented") }
        }

        turn = make_move(sender.clone(), &receiver, turn, &stream, &mut game)
    }
}

fn make_move(sender: Sender<ServerToGame>, receiver: &Receiver<GameToServer>, turn: Color, stream: &TcpStream, game: &mut Game) -> Color {
    let move_made = receiver.recv().unwrap();

    match game.do_move(move_made.move_made.into_chess()) {
        Ok(d) => {
            sender.send(ServerToGame::State {
                game: game.clone(),
                move_made: move_made.move_made,
                turn: turn.clone(),
                joever: d.into_network(),
            }).unwrap();

            let state = ServerToClient::State {
                board: game.board.into_network(),
                moves: vec![],
                joever: d.into_network(),
                move_made: move_made.move_made,
            };

            //send
            serde_json::to_writer(stream, &state).unwrap();

            switch_turn(turn)
        }
        Err(e) => {
            sender.send(ServerToGame::Error {
                error: e,
            }).unwrap();
            make_move(sender, receiver, turn, stream, game)
        }
    }
}