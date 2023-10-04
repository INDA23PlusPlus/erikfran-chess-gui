use serde::Deserialize;
use chess_network_protocol::*;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Sender, Receiver};

use redkar_chess::Game;
use crate::{redkar_chess_utils::*, TcpToGame};

pub const FEATURES: Vec<Features> = vec![];

pub fn run(sender: Sender<TcpToGame>, receiver: Receiver<Move>) {
    let listener = TcpListener::bind("127.0.0.1:5000").unwrap();

    // accept connections and process them serially
    let (stream, _addr) = listener.accept().unwrap();
    let mut de = serde_json::Deserializer::from_reader(&stream);

    //receive
    let deserialized = ClientToServerHandshake::deserialize(&mut de).unwrap();

    let mut game = Game::new_game();

    sender.send(TcpToGame::Handshake {
        board: game.board.into_network(),
        moves: vec![],
        features: FEATURES,
        server_color: deserialized.server_color.clone(),
    }).unwrap();

    let handshake = ServerToClientHandshake {
        features: FEATURES,
        board: game.board.into_network(),
        moves: vec![],
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

fn client_move(sender: &Sender<TcpToGame>, receiver: &Receiver<Move>, stream: &TcpStream, game: &mut Game) {
    let mut de = serde_json::Deserializer::from_reader(stream);

    //receive
    let deserialized = ClientToServer::deserialize(&mut de).unwrap();

    match deserialized {
        ClientToServer::Move(move_made) => {
            match game.do_move(move_made.into_chess()) {
                Ok(d) => {
                    sender.send(TcpToGame::State {
                        board: game.board.into_network(),
                        moves: vec![],
                        turn: game.turn.into_network(),
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
                }
                Err(e) => {
                    let state = ServerToClient::Error {
                        board: game.board.into_network(),
                        moves: vec![],
                        joever: Joever::Ongoing,
                        message: crate::explain_move_error(e),
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

fn make_move(sender: &Sender<TcpToGame>, receiver: &Receiver<Move>, stream: &TcpStream, game: &mut Game) {
    let move_made = receiver.recv().unwrap();

    match game.do_move(move_made.into_chess()) {
        Ok(d) => {
            sender.send(TcpToGame::State {
                board: game.board.into_network(),
                moves: vec![],
                turn: game.turn.into_network(),
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
        }
        Err(e) => {
            sender.send(TcpToGame::Error { message: explain_move_error(e) }).unwrap();
            make_move(sender, receiver, stream, game);
        }
    }
}