use serde::Deserialize;
use chess_network_protocol::*;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Sender, Receiver};

use crate::TcpToGame;

use local_ip_address::local_ip;

pub trait UniversalGame {
    fn try_move(&mut self, m: Move) -> Result<(), String>;
    fn possible_moves(&self) -> Vec<Move>;
    fn new() -> Self;
    fn board(&self) -> [[Piece; 8]; 8];
    fn turn(&self) -> Color;
    fn joever(&self) -> Joever;
    fn features(&self) -> Vec<Features>;
}

pub fn run(sender: Sender<TcpToGame>, receiver: Receiver<Move>, mut game: impl UniversalGame) {
    let listener = TcpListener::bind(local_ip().unwrap().to_string() + ":8384").unwrap();

    // accept connections and process them serially
    let (stream, _addr) = listener.accept().unwrap();
    let mut de = serde_json::Deserializer::from_reader(&stream);

    //receive
    let deserialized = ClientToServerHandshake::deserialize(&mut de).unwrap();

    let moves = game.possible_moves();

    sender.send(TcpToGame::Handshake {
        board: game.board(),
        moves: moves.clone(),
        features: game.features(),
        server_color: deserialized.server_color.clone(),
    }).unwrap();

    let handshake = ServerToClientHandshake {
        features: game.features(),
        board: game.board(),
        moves: moves,
        joever: Joever::Ongoing,
    };

    //send
    serde_json::to_writer(&stream, &handshake).unwrap();

    if &deserialized.server_color == &Color::White {
        make_move(&sender, &receiver, &stream, &mut game)
    }

    loop {
        client_move(&sender, &receiver, &stream, &mut game);

        make_move(&sender, &receiver, &stream, &mut game);
    }
}

fn client_move(sender: &Sender<TcpToGame>, receiver: &Receiver<Move>, stream: &TcpStream, game: &mut impl UniversalGame) {
    let mut de = serde_json::Deserializer::from_reader(stream);

    //receive
    let deserialized = ClientToServer::deserialize(&mut de).unwrap();

    match deserialized {
        ClientToServer::Move(move_made) => {
            match game.try_move(move_made) {
                Ok(()) => {
                    let moves = game.possible_moves();
                    sender.send(TcpToGame::State {
                        board: game.board(),
                        moves: moves.clone(),
                        turn: game.turn(),
                        move_made: move_made,
                        joever: game.joever(),
                    }).unwrap();

                    let state = ServerToClient::State {
                        board: game.board(),
                        moves: moves,
                        joever: game.joever(),
                        move_made: move_made,
                    };

                    //send
                    serde_json::to_writer(stream, &state).unwrap();
                }
                Err(e) => {
                    let state = ServerToClient::Error {
                        board: game.board(),
                        moves: game.possible_moves(),
                        joever: Joever::Ongoing,
                        message: e,
                    };

                    //send
                    serde_json::to_writer(stream, &state).unwrap();

                    client_move(sender, receiver, stream, game)
                }
            }
        },
        ClientToServer::Resign => { panic!("Resign not implemented") }
        ClientToServer::Draw => { panic!("Draw not implemented") }
    }
}

fn make_move(sender: &Sender<TcpToGame>, receiver: &Receiver<Move>, stream: &TcpStream, game: &mut impl UniversalGame) {
    let move_made = receiver.recv().unwrap();

    match game.try_move(move_made) {
        Ok(()) => {
            let moves = game.possible_moves();
            sender.send(TcpToGame::State {
                board: game.board(),
                moves: moves.clone(),
                turn: game.turn(),
                move_made: move_made,
                joever: game.joever(),
            }).unwrap();

            let state = ServerToClient::State {
                board: game.board(),
                moves: moves,
                joever: game.joever(),
                move_made: move_made,
            };

            //send
            serde_json::to_writer(stream, &state).unwrap();
        }
        Err(message) => {
            sender.send(TcpToGame::Error { message }).unwrap();
            make_move(sender, receiver, stream, game);
        }
    }
}