#![feature(let_chains)]
use ggez::event::MouseButton;
use ggez::{event, conf};
use ggez::graphics::{self, Rect, Text, PxScale, DrawParam};
use ggez::{Context, GameResult, glam};
use ggez::glam::*;

use chess_network_protocol;

mod redkar_chess_utils;

use std::sync::mpsc::{Receiver, Sender};
use std::{env, path, thread};

use ggegui::{egui, Gui};

use redkar_chess_utils::*;
use chess_network_protocol::*;

const SCALE: f32 = 0.75;
const SQUARE_SIZE: f32 = 130.0 * SCALE;
const TEXT_SIZE: f32 = 25.0 * SCALE;
const SIDEBAR_SIZE: f32 = 400.0;
const UI_SCALE: f32 = 10.0;
const FONT_SIZE: f32 = 32.0;

pub enum TcpToGame {
    Handshake {
        board: [[Piece; 8]; 8],
        moves: Vec<Move>,
        features: Vec<Features>,
        server_color: Color,
    },
    State {
        board: [[Piece; 8]; 8],
        moves: Vec<Move>,
        joever: Joever,
        move_made: Move,
        turn: Color,
    },
    Error {
        message: String,
    },
}



struct MainState {
    pawn_image_w: graphics::Image,
    pawn_image_b: graphics::Image,
    king_image_w: graphics::Image,
    king_image_b: graphics::Image,
    queen_image_w: graphics::Image,
    queen_image_b: graphics::Image,
    bishop_image_w: graphics::Image,
    bishop_image_b: graphics::Image,
    knight_image_w: graphics::Image,
    knight_image_b: graphics::Image,
    rook_image_w: graphics::Image,
    rook_image_b: graphics::Image,
    selected: Option<Vec2>,
    start_x: f32,
    start_y: f32,
    pos_x: f32,
    pos_y: f32,
    last_move: Option<Move>,
    controls_text: graphics::Text,
    text: Text,
    joever: Joever,
    gui: Gui,
    is_server: Option<bool>,
    server_color: Option<chess_network_protocol::Color>,
    tcp_started: bool,
    receiver: Option<Receiver<TcpToGame>>,
    sender: Option<Sender<Move>>,
    board: [[Piece; 8]; 8],
    moves: Vec<Move>,
    features: Vec<Features>,
    turn: Color,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let pawn_image_w = graphics::Image::from_path(ctx, "/pawn-w.png")?;
        let pawn_image_b = graphics::Image::from_path(ctx, "/pawn-b.png")?;
        let king_image_w = graphics::Image::from_path(ctx, "/king-w.png")?;
        let king_image_b = graphics::Image::from_path(ctx, "/king-b.png")?;
        let queen_image_w = graphics::Image::from_path(ctx, "/queen-w.png")?;
        let queen_image_b = graphics::Image::from_path(ctx, "/queen-b.png")?;
        let bishop_image_w = graphics::Image::from_path(ctx, "/bishop-w.png")?;
        let bishop_image_b = graphics::Image::from_path(ctx, "/bishop-b.png")?;
        let knight_image_w = graphics::Image::from_path(ctx, "/knight-w.png")?;
        let knight_image_b = graphics::Image::from_path(ctx, "/knight-b.png")?;
        let rook_image_w = graphics::Image::from_path(ctx, "/rook-w.png")?;
        let rook_image_b = graphics::Image::from_path(ctx, "/rook-b.png")?;

        let mut controls_text = Text::new("Controls:\n\nHold left click and drag to move a piece and just release left click on the destination square to make the move.");
        controls_text.set_scale(PxScale::from(TEXT_SIZE));

        let mut gui =  Gui::new(ctx);

        /*let mut style = (*gui.ctx().style()).clone();
        style.text_styles = [
            (Heading, FontId::new(FONT_SIZE, Proportional)),
            (Body, FontId::new(FONT_SIZE, Proportional)),
            (Monospace, FontId::new(FONT_SIZE, Proportional)),
            (Button, FontId::new(FONT_SIZE, Proportional)),
            (Small, FontId::new(FONT_SIZE, Proportional)),
        ]
        .into();
        gui.ctx().set_style(style);*/

        let s = MainState {
            pawn_image_w,
            pawn_image_b,
            king_image_w,
            king_image_b,
            queen_image_w,
            queen_image_b,
            bishop_image_w,
            bishop_image_b,
            knight_image_w,
            knight_image_b,
            rook_image_w,
            rook_image_b,
            selected: None,
            start_x: 0.0,
            start_y: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
            last_move: None,
            controls_text,
            text: Text::new(""),
            joever: Joever::Ongoing,
            gui,
            is_server: None,
            server_color: None,
            tcp_started: false,
            receiver: None,
            sender: None,
            board: [[Piece::None; 8]; 8],
            moves: vec![],
            features: vec![],
            turn: Color::White,
        };

        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
	fn update(&mut self, ctx: &mut Context) -> GameResult {
        let gui_ctx = self.gui.ctx();

        if self.tcp_started && let Some(receiver) = &self.receiver {
            if let Ok(message) = receiver.try_recv() {
                match message {
                    TcpToGame::Handshake { .. } => unreachable!(),
                    TcpToGame::State { board, moves, joever, move_made, turn } => {
                        self.board = board;
                        self.moves = moves;
                        self.last_move = Some(move_made);
                        self.joever = joever;
                        self.turn = turn;
                        self.text = Text::new(
                            format!("{:?} moved {:?} from {} to {}",
                                self.turn, 
                                self.board[move_made.end_y][move_made.end_x],
                                cords_to_square(move_made.start_x as f32, move_made.start_y as f32), 
                                cords_to_square(move_made.end_x as f32, move_made.end_y as f32)
                            ));
                    },
                    TcpToGame::Error { .. } => unreachable!(),
                }
            }
        }
        if let Some(receiver) = &self.receiver && !self.tcp_started {
            if let Ok(message) = receiver.try_recv() {
                match message {
                    TcpToGame::Handshake { board, moves, features, server_color } => {
                        self.board = board;
                        self.moves = moves;
                        self.features = features;
                        self.server_color = Some(server_color);
                        self.tcp_started = true;
                    },
                    TcpToGame::State { .. } => unreachable!(),
                    TcpToGame::Error { .. } => unreachable!(),
                }
            }
            println!("{:?}", self.server_color);
            if self.is_server.unwrap() {
                egui::Area::new("").show(&gui_ctx, |ui| {
                    ui.label("Waiting for client to connect...");
                });
            }
        }
        else if self.receiver.is_none() {
            egui::Area::new("").show(&gui_ctx, |ui| {
                ui.label("Want to start a session as server or client?");
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.is_server, 
                        Some(true), 
                        "Server"
                    );
                    ui.selectable_value(
                        &mut self.is_server, 
                        Some(false), 
                        "Client"
                    );
                });
    
                if Some(false) == self.is_server {
                    ui.label("Want color do you want to play as?");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.server_color, 
                            Some(Color::Black), 
                            "White"
                        );
                        ui.selectable_value(
                            &mut self.server_color, 
                            Some(Color::White), 
                            "Black"
                        );
                    });
                }
    
                ui.add_enabled_ui(
                    self.server_color.is_some() || Some(true) == self.is_server, 
                    |ui| {
                        if ui.button("Connect").clicked() {
                            let (tcp_sender, tcp_receiver) = std::sync::mpsc::channel();
                            let (game_sender, game_receiver) = std::sync::mpsc::channel();
    
                            if Some(true) == self.is_server {
                                thread::spawn(move || server::run(tcp_sender, game_receiver));
                            } else {
                                let temp = self.server_color.unwrap();
    
                                thread::spawn(move || client::run(
                                    tcp_sender, 
                                    game_receiver, 
                                    temp));
                            }
    
                            self.receiver = Some(tcp_receiver);
                            self.sender = Some(game_sender);
                        }
                });
            });
        }

		self.gui.update(ctx);
		Ok(())
	}

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::Color::BLACK,
        );

        if self.tcp_started && let Some(is_server) = self.is_server && let Some(server_color) = &self.server_color {

            if your_turn(&self.turn, server_color, is_server) {
                println!("Your turn! is_server: {}, server_color: {:?}, turn: {:?}", is_server, server_color, self.turn);
            }
            else {
                println!("Opponents turn! is_server: {}, server_color: {:?}, turn: {:?}", is_server, server_color, self.turn);
            }

            let white_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                graphics::Color::WHITE,
            )?;
            let black_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                graphics::Color::from_rgb(180, 135, 103,),
            )?;
            let white_selected_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                graphics::Color::from_rgb(207, 209, 134)
            )?;
            let black_selected_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                graphics::Color::from_rgb(170, 162, 87)
            )?;

            let mut selected_image: &graphics::Image = &self.pawn_image_w;
            let mut relative_pos: Vec2 = Vec2::new(0.0, 0.0);

            let (last_move_pos_from, last_move_pos_to) = match self.last_move {
                Some(mv) => (Vec2::new(mv.start_x as f32, mv.start_y as f32), Vec2::new(mv.end_x as f32, mv.end_y as f32)),
                None => (Vec2::new(-1.0, -1.0), Vec2::new(-1.0, -1.0)),
            };

            for x in 0..8 {
                for y in 0..8 {
                    let y_c = y_colored(is_server, server_color, y);
                    let pos = Vec2::new(x as f32 * SQUARE_SIZE, y_c as f32 * SQUARE_SIZE);
                    let pos_unit = Vec2::new(x as f32, y as f32);
                    let selected = 
                        (self.selected == Some(pos_unit) 
                            && self.board[y][x] != Piece::None)
                        || (pos_unit == last_move_pos_from 
                            || pos_unit == last_move_pos_to);
                    
                    if (x + y_c) % 2 == 0 {
                        if selected {
                            canvas.draw(&black_selected_square, pos);
                        }
                        else {
                            canvas.draw(&black_square, pos);
                        }
                    } 
                    else {
                        if selected {
                            canvas.draw(&white_selected_square, pos);
                        }
                        else {
                            canvas.draw(&white_square, pos);
                        }
                    }

                    let image = match self.board[y][x] {
                        Piece::WhitePawn => &self.pawn_image_w,
                        Piece::BlackPawn => &self.pawn_image_b,
                        Piece::WhiteKing => &self.king_image_w,
                        Piece::BlackKing => &self.king_image_b,
                        Piece::WhiteQueen => &self.queen_image_w,
                        Piece::BlackQueen => &self.queen_image_b,
                        Piece::WhiteBishop => &self.bishop_image_w,
                        Piece::BlackBishop => &self.bishop_image_b,
                        Piece::WhiteKnight => &self.knight_image_w,
                        Piece::BlackKnight => &self.knight_image_b,
                        Piece::WhiteRook => &self.rook_image_w,
                        Piece::BlackRook => &self.rook_image_b,
                        Piece::None => continue,
                    };

                    if self.selected == Some(pos_unit) && self.board[y][x] != Piece::None {
                        if your_turn(&self.turn, server_color, is_server) &&  {
                            relative_pos = pos + Vec2::new(self.pos_x - self.start_x, self.pos_y - self.start_y);
                        }
                        else {
                            relative_pos = pos
                        }

                        selected_image = image;
                    }
                    else {
                        canvas.draw(image, graphics::DrawParam::new()
                            .dest(pos)
                            .scale(Vec2::new(0.75, 0.75)));
                    }
                }
            }

            if let Some(selected) = self.selected {
                canvas.draw(selected_image, graphics::DrawParam::new()
                    .dest(relative_pos)
                    .scale(Vec2::new(0.75, 0.75)));
            }

            let _ = &self.text.set_scale(TEXT_SIZE)
                .set_bounds(Vec2::new(SIDEBAR_SIZE - TEXT_SIZE * 2.0, f32::INFINITY))
                .set_wrap(true);
            
            let _ = &self.controls_text.set_bounds(Vec2::new(SIDEBAR_SIZE - TEXT_SIZE * 2.0, f32::INFINITY))
            .set_wrap(true);

            let controls_text_pos = Vec2::new(8.0 * SQUARE_SIZE + TEXT_SIZE, TEXT_SIZE);
            let text_pos = controls_text_pos + 
                Vec2::new( 0.0, &self.controls_text.measure(ctx).unwrap().y + TEXT_SIZE);


            canvas.draw(&self.controls_text, controls_text_pos);
            canvas.draw(&self.text, text_pos);

            if &self.joever != &Joever::Ongoing {
                let mut text = Text::new("");
                match &self.joever {
                    Joever::Black => {
                        text = Text::new("Black won!");
                    },
                    Joever::White => {
                        text = Text::new("White won!");
                    },
                    Joever::Draw => {
                        text = Text::new("Draw!");
                    },
                    Joever::Ongoing => { },
                    Joever::Indeterminate => {
                        text = Text::new("Indeterminate!");
                    },
                }
                
                let _ = text.set_scale(PxScale::from(55.0));
                let text_pos = Vec2::new(3.0 * SQUARE_SIZE - 100.0, 3.0 * SQUARE_SIZE);
                canvas.draw(&text, text_pos);

            }

        }

        canvas.draw(&self.gui, DrawParam::default()
            .dest(glam::Vec2::ZERO)
            .scale([UI_SCALE, UI_SCALE]));

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    ) -> GameResult {
        self.pos_x = x;
        self.pos_y = y;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if self.tcp_started && let Some(is_server) = self.is_server && let Some(server_color) = &self.server_color && x < 8.0 * SQUARE_SIZE && y < 8.0 * SQUARE_SIZE {
            let y_c = y_colored(is_server, server_color, (y as f32 / SQUARE_SIZE).floor() as usize);
            let temp = Some(Vec2::new((x / SQUARE_SIZE).floor(), y_c as f32));

            if self.joever != Joever::Ongoing {
                self.selected = None;
                return Ok(());
            }
            
            self.start_x = x;
            self.start_y = y;
            self.selected = temp;
            return Ok(());
        }
        self.selected = None;
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if self.joever != Joever::Ongoing 
            || !self.tcp_started 
            || x > 8.0 * SQUARE_SIZE 
            || y > 8.0 * SQUARE_SIZE 
            || self.selected.is_none() 
        {
            self.selected = None;
            return Ok(());
        }

        if let Some(is_server) = self.is_server && let Some(server_color) = &self.server_color && let Some(selected) = self.selected {
            let y_c = y_colored(is_server, server_color, (y as f32 / SQUARE_SIZE).floor() as usize);

            let mv = Move { 
                start_x: selected.x as usize, 
                start_y: selected.y as usize, 
                end_x: (x / SQUARE_SIZE).floor() as usize, 
                end_y: y_c,
                promotion: Piece::None,
            };

            if !your_turn(&self.turn, server_color, is_server) {
                return Ok(());
            }

            if let Some(sender) = &self.sender {
                sender.send(mv).unwrap();
            }
            if let Some(receiver) = &self.receiver {
                match receiver.recv().unwrap() {
                    TcpToGame::State { board, moves, joever, move_made, turn } => {
                        self.board = board;
                        self.last_move = Some(move_made);
                        self.joever = joever;
                        self.text = Text::new(
                            format!("{:?} moved {:?} from {} to {}",
                                self.turn, 
                                self.board[selected.y as usize][selected.x as usize],
                                cords_to_square(selected.x, selected.y), 
                                cords_to_square((x / SQUARE_SIZE).floor(), (y / SQUARE_SIZE).floor()
                            )));
                    },
                    TcpToGame::Error { message } => {
                        self.text = Text::new(
                            format!("Move error: {}", message))
                    },
                    TcpToGame::Handshake { .. } => unreachable!(),
                }
            }
            self.selected = None;
        }
        Ok(())
    }
}

fn your_turn(turn: &Color, server_color: &Color, is_server: bool) -> bool {
    (turn != server_color) ^ is_server
}

fn oposite_color(color: &Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

fn piece_color(piece: &Piece) -> Option<Color> {
    match piece {
        Piece::WhitePawn => Some(Color::White),
        Piece::BlackPawn => Some(Color::Black),
        Piece::WhiteKing => Some(Color::White),
        Piece::BlackKing => Some(Color::Black),
        Piece::WhiteQueen => Some(Color::White),
        Piece::BlackQueen => Some(Color::Black),
        Piece::WhiteBishop => Some(Color::White),
        Piece::BlackBishop => Some(Color::Black),
        Piece::WhiteKnight => Some(Color::White),
        Piece::BlackKnight => Some(Color::Black),
        Piece::WhiteRook => Some(Color::White),
        Piece::BlackRook => Some(Color::Black),
        Piece::None => None,
    }
}

fn y_colored(is_server: bool, server_color: &chess_network_protocol::Color, y: usize) -> usize {
    match server_color {
        chess_network_protocol::Color::White => 
        if !is_server { y } 
        else { 7 - y },
        chess_network_protocol::Color::Black => 
        if !is_server { 7 - y } 
        else { y },
    }
}

fn cords_to_square(x: f32, y: f32) -> String {
    let t = match x as u8 {
        0 => "a".to_string(),
        1 => "b".to_string(),
        2 => "c".to_string(),
        3 => "d".to_string(),
        4 => "e".to_string(),
        5 => "f".to_string(),
        6 => "g".to_string(),
        7 => "h".to_string(),
        _ => unreachable!(),
    };

    t + y.to_string().as_str()
}

mod client;
mod server;

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("chess", "Erik Frankling")
    .add_resource_path(resource_dir)
    .window_mode(
        conf::WindowMode::default()
            .dimensions(SQUARE_SIZE * 8.0 + SIDEBAR_SIZE, SQUARE_SIZE * 8.0),
    );
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state);
}
