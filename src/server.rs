use serde::{Serialize, Deserialize};
use chess_network_protocol::*;

use std::net::TcpListener;
use std::sync::mpsc::{Sender, Receiver};

use redkar_chess::Game;

pub const FEATURES: Vec<Features> = vec![];

pub struct ServerToGame {
    pub game: Game,
    pub server_color: Color,
    pub move_made: Move,
}

pub struct GameToServer {
    pub move_made: Move,
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

    sender.send(ServerToGame {
        game: game,
        server_color: deserialized.server_color,
        move_made: Move {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 0,
            promotion: Piece::None,
        },
    }).unwrap();

    let handshake = ServerToClientHandshake {
        features: FEATURES,
        board: [[Piece::BlackBishop; 8]; 8],
        moves: vec![Move {
            start_x: 0,
            start_y: 0,
            end_x: 1,
            end_y: 1,
            promotion: Piece::None,
        }],
        joever: Joever::Ongoing,
    };

    //send
    serde_json::to_writer(&stream, &handshake).unwrap();

    //assumes that the client is white
    //receive
    let deserialized = ClientToServer::deserialize(&mut de).unwrap();
    println!("Received: {:?}", deserialized);

    let state = ServerToClient::State {
        board: [[Piece::BlackBishop; 8]; 8],
        moves: vec![Move {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 0,
            promotion: Piece::None,
        }],
        joever: Joever::Ongoing,
        //should be the move recieved from the client
        move_made: Move {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 0,
            promotion: Piece::None,
        }
    };
    
    //send
    serde_json::to_writer(&stream, &state).unwrap();

    let state = ServerToClient::State {
        board: [[Piece::BlackBishop; 8]; 8],
        moves: vec![Move {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 0,
            promotion: Piece::None,
        }],
        joever: Joever::Ongoing,
        //should be the move made by the server
        move_made: Move {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 0,
            promotion: Piece::None,
        }
    };

    //send
    serde_json::to_writer(&stream, &state).unwrap();
}