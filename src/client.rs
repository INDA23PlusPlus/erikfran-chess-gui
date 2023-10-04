use serde::{Serialize, Deserialize};
use chess_network_protocol::*;
use serde_json::de::IoRead;

use std::net::TcpStream;
use std::sync::mpsc::{Sender, Receiver};

use crate::TcpToGame;

fn switch_turn(turn: &Color) -> Color {
    match turn {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

pub fn run(sender: Sender<TcpToGame>, receiver: Receiver<Move>, server_color: Color) {
    let stream = TcpStream::connect("127.0.0.1:5000").unwrap();
    let mut de = serde_json::Deserializer::from_reader(&stream);

    let handshake = ClientToServerHandshake {
        server_color: server_color.clone(),
    };

    //send
    serde_json::to_writer(&stream, &handshake).unwrap();

    //receive
    let deserialized = ServerToClientHandshake::deserialize(&mut de).unwrap();
    println!("Recieved: {:?}", deserialized);

    let mut turn = switch_turn(&server_color);

    sender.send(TcpToGame::Handshake {
        board: deserialized.board,
        moves: deserialized.moves,
        features: deserialized.features,
        server_color: server_color.clone(),
    }).unwrap();

    if &turn == &Color::White {
        turn = make_move(sender.clone(), &receiver, turn, &stream);
    }

    loop {
        let deserialized = ServerToClient::deserialize(&mut de).unwrap();
        println!("Recieved: {:?}", deserialized);

        match deserialized {
            ServerToClient::State { board, moves, joever, move_made } => {
                turn = switch_turn(&turn);

                sender.send(TcpToGame::State { 
                    board, 
                    moves, 
                    joever, 
                    move_made, 
                    turn: turn.clone(),
                }).unwrap();
            },
            ServerToClient::Error { .. } => { panic!("Error cant happen here") },
            ServerToClient::Draw { board, moves } => {panic!("Draw not implemented")},
            ServerToClient::Resigned { board, joever } => {panic!("Resigned not implemented")},
        }

        turn = make_move(sender.clone(), &receiver, turn, &stream);
    }
}

fn make_move(sender: Sender<TcpToGame>, receiver: &Receiver<Move>, turn: Color, stream: &TcpStream) -> Color {
    let mut de = serde_json::Deserializer::from_reader(stream);
    
    let move_made = receiver.recv().unwrap();

    let mv = ClientToServer::Move(move_made);

    //send
    serde_json::to_writer(stream, &mv).unwrap();

    let deserialized = ServerToClient::deserialize(&mut de).unwrap();
    println!("Recieved: {:?}", deserialized);

    match deserialized {
        ServerToClient::State { board, moves, joever, move_made } => {
            sender.send(TcpToGame::State { 
                board, 
                moves, 
                joever, 
                move_made, 
                turn: switch_turn(&turn),
            }).unwrap();
            
            return switch_turn(&turn);
        },
        ServerToClient::Error { board, moves, joever, message } => {
            sender.send(TcpToGame::Error { message }).unwrap();

            return make_move(sender, receiver, turn, stream);
        },
        ServerToClient::Draw { board, moves } => {panic!("Draw not implemented")},
        ServerToClient::Resigned { board, joever } => {panic!("Resigned not implemented")},
    }
}