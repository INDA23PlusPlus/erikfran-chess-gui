#![feature(let_chains)]
use ggegui::egui::{TextBuffer, Mesh};
use ggez::event::MouseButton;
use ggez::{event, conf};
use ggez::graphics::{self, Rect, Text, PxScale, DrawParam, TextFragment};
use ggez::{Context, GameResult, glam};
use ggez::glam::*;

use chess_network_protocol;
use server::UniversalGame;

mod redkar_chess_utils;
mod fritiofr_chess_utils;
mod erikfran_chess_utils;

use std::f32::consts::PI;
use std::sync::mpsc::{Receiver, Sender};
use std::{env, path, thread, cmp::Ord};

use ggegui::{egui, Gui};

use chess_network_protocol::*;

use local_ip_address::local_ip;

const SCALE: f32 = 0.75;
const SQUARE_SIZE: f32 = 130.0 * SCALE;
const TEXT_SIZE: f32 = 25.0 * SCALE;
const SIDEBAR_SIZE: f32 = 500.0 * SCALE;
const UI_SCALE: f32 = 1.0;
const FONT_SIZE: f32 = 32.0;
const DRAG_SENSITIVITY: f32 = 25.0 * SCALE;
const CORD_OFFSET: f32 = 35.0;
const CORD_FONT_SIZE: f32 = 30.0;
const MOVE_RADIUS: f32 = 25.0 * SCALE;
const MOVE_CAPTURE_SIZE: f32 = 25.0 * SCALE;

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Redkar,
    Fritiofr,
    Erikfran,
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
    white_rgb: graphics::Color,
    black_rgb: graphics::Color,
    move_future_rgb_white: graphics::Color,
    move_future_rgb_black: graphics::Color,
    move_rgb_white: graphics::Color,
    move_rgb_black: graphics::Color,
    white_square: graphics::Mesh,
    black_square: graphics::Mesh,
    white_moved_square: graphics::Mesh,
    black_moved_square: graphics::Mesh,
    white_selected_square: graphics::Mesh,
    black_selected_square: graphics::Mesh,
    white_moving_square: graphics::Mesh,
    black_moving_square: graphics::Mesh,
    selected: Option<Vec2>,
    dragging: bool,
    start_x: f32,
    start_y: f32,
    pos_x: f32,
    pos_y: f32,
    last_move: Option<Move>,
    controls_text: String,
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
    ip: String,
    move_circle: graphics::Mesh,
    move_capture: graphics::Mesh,
    backend: Backend,
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

        let white_rgb: graphics::Color = graphics::Color::from_rgb(240, 217, 181);
        let black_rgb: graphics::Color = graphics::Color::from_rgb(180, 135, 103);

        let move_future_rgb_white = graphics::Color::from_rgb(129,123,132);
        let move_future_rgb_black = graphics::Color::from_rgb(100,82,92);
        let move_rgb_white = graphics::Color::from_rgb(129,150,105);
        let move_rgb_black = graphics::Color::from_rgb(129,150,105);

        let white_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            white_rgb,
        )?;
        let black_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            black_rgb,
        )?;
        let white_moved_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            graphics::Color::from_rgb(207, 209, 134)
        )?;
        let black_moved_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            graphics::Color::from_rgb(170, 162, 87)
        )?;
        let white_selected_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            graphics::Color::from_rgb(129,150,105)
        )?;
        let black_selected_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            graphics::Color::from_rgb(100,109,64)
        )?;
        let white_moving_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            graphics::Color::from_rgb(174, 177, 136)
        )?;
        let black_moving_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            graphics::Color::from_rgb(133, 120, 78)
        )?;
        let black_moving_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            graphics::Color::from_rgb(133, 120, 78)
        )?;
        
        let move_circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            MOVE_RADIUS,
            0.1,
            move_rgb_white,
        )?;
        let move_capture = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::fill(),
            &[Vec2::new(0.0, 0.0), Vec2::new(0.0, MOVE_CAPTURE_SIZE), Vec2::new(MOVE_CAPTURE_SIZE, 0.0)],
            move_rgb_white,
        )?;

        let mut controls_text = "Controls:\n\nHold left click and drag to move a piece and just release left click on the destination square to make the move.".to_string();

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
            white_rgb,
            black_rgb,
            move_future_rgb_white,
            move_future_rgb_black,
            move_rgb_white,
            move_rgb_black,
            white_square,
            black_square,
            white_moved_square,
            black_moved_square,
            white_selected_square,
            black_selected_square,
            white_moving_square,
            black_moving_square,
            selected: None,
            dragging: false,
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
            ip: local_ip().unwrap().to_string(),
            move_circle,
            move_capture,
            backend: Backend::Redkar,
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
                        self.text = Text::new(
                            format!("{:?} moved {:?} from {} to {}",
                                self.turn, 
                                self.board[move_made.end_y][move_made.end_x],
                                cords_to_square(move_made.start_x as f32, move_made.start_y as f32), 
                                cords_to_square(move_made.end_x as f32, move_made.end_y as f32)
                            ));
                        self.turn = turn;
                        self.selected = None;
                        self.dragging = false;
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

                        let mut features_text = "Features: ".to_string();

                        for f in &features {
                            features_text += match f {
                                Features::Castling => &"Castling, ",
                                Features::EnPassant => "En Passant, ",
                                Features::Promotion => "Promotion, ",
                                Features::PossibleMoveGeneration => "Possible Move Generation, ",
                                Features::Stalemate => "Stalemate, ",
                                Features::Other(f) => f.as_str(),
                            }
                        }

                        self.controls_text = self.controls_text.clone() + "\n\n" + features_text.as_str();

                        self.features = features;
                        self.server_color = Some(server_color);
                        self.tcp_started = true;
                    },
                    TcpToGame::State { .. } => unreachable!(),
                    TcpToGame::Error { .. } => unreachable!(),
                }
            }

            if self.is_server.unwrap() {
                egui::Area::new("").movable(false).show(&gui_ctx, |ui| {
                    ui.label("Waiting for client to connect... \n Your ip: ".to_string() + local_ip().unwrap().to_string().as_str() + ":8384".as_str());
                });
            }
        }
        else if self.receiver.is_none() {
            egui::Area::new("").movable(false).show(&gui_ctx, |ui| {
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

                if Some(true) == self.is_server {
                    ui.label("Want backend do you want to use?");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.backend, 
                            Backend::Redkar, 
                            "Redkar"
                        );
                        ui.selectable_value(
                            &mut self.backend, 
                            Backend::Fritiofr, 
                            "Fritiofr"
                        );
                        ui.selectable_value(
                            &mut self.backend, 
                            Backend::Erikfran, 
                            "Erikfran"
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Your ip: ".to_string() + local_ip().unwrap().to_string().as_str() + ":8384".as_str());
                        
                        if ui.button("Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = local_ip().unwrap().to_string() + ":8384".as_str());
                        }
                    });
                }
    
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
                    
                    ui.label("What is the IP of your opponent?");
                    
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.ip).hint_text(local_ip().unwrap().to_string()));
                        
                        if ui.button("Paste").clicked() {
                            ui.output(|o| {println!("{}", o.copied_text.as_str().to_string());  self.ip = (&o.copied_text).to_string()});
                            println!("{}", self.ip);
                        }
                    });
                }
    
                ui.add_enabled_ui(
                    self.server_color.is_some() || Some(true) == self.is_server, 
                    |ui| {
                        if ui.button("Connect").clicked() {
                            let (tcp_sender, tcp_receiver) = std::sync::mpsc::channel();
                            let (game_sender, game_receiver) = std::sync::mpsc::channel();
    
                            if Some(true) == self.is_server {
                                match self.backend {
                                    Backend::Fritiofr => thread::spawn(move || server::run(tcp_sender, game_receiver, crate::fritiofr_chess_utils::Game::new())),
                                    Backend::Redkar => thread::spawn(move || server::run(tcp_sender, game_receiver, crate::redkar_chess_utils::Game::new())),
                                    Backend::Erikfran => thread::spawn(move || server::run(tcp_sender, game_receiver, crate::erikfran_chess_utils::Game::new())),
                                };

                                
                            } else {
                                let temp = self.server_color.clone().unwrap();
                                let temp_ip = self.ip.clone();
    
                                thread::spawn(move || client::run(
                                    tcp_sender, 
                                    game_receiver, 
                                    temp,
                                    temp_ip.to_string()));
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
            let mut moves = [[false; 8]; 8];

            if let Some(pos) = self.selected {
                for m in &self.moves {
                    if m.start_x == pos.x as usize && m.start_y == pos.y as usize {
                        moves[m.end_y][m.end_x] = true;
                    }
                }
            }

            let mut selected_image: Option<&graphics::Image> = None;

            let (last_move_pos_from, last_move_pos_to) = match self.last_move {
                Some(mv) => (Vec2::new(mv.start_x as f32, mv.start_y as f32), Vec2::new(mv.end_x as f32, mv.end_y as f32)),
                None => (Vec2::new(-1.0, -1.0), Vec2::new(-1.0, -1.0)),
            };

            for x in 0..8 {
                for y in 0..8 {
                    let y_c = y_colored(is_server, server_color, y);
                    let x_c = x_colored(is_server, server_color, x);
                    let pos = Vec2::new(x_c as f32 * SQUARE_SIZE, y_c as f32 * SQUARE_SIZE);
                    let pos_unit = Vec2::new(x as f32, y as f32);
                    let selected = 
                        self.selected == Some(pos_unit);

                    let moved = 
                        pos_unit == last_move_pos_from 
                        || pos_unit == last_move_pos_to;
                    
                    let moving = 
                        x_colored(is_server, server_color, (self.pos_x / SQUARE_SIZE).floor() as usize) == pos_unit.x as usize
                        && y_colored(is_server, server_color, (self.pos_y / SQUARE_SIZE).floor() as usize) == pos_unit.y as usize
                        && self.selected.is_some()
                        && piece_color(&self.board[y][x]) != Some(your_color(server_color, is_server))
                        && your_turn(&self.turn, server_color, is_server);

/*                     let text_pos_y = Vec2::new((x as f32 + 1.0) * SQUARE_SIZE - CORD_OFFSET, 8.0 * SQUARE_SIZE - CORD_OFFSET);
                    let text_pos_x = Vec2::new(0.0, y as f32 * SQUARE_SIZE); */

                    let mut move_color = None;
                    
                    if (x_c + y_c) % 2 == 0 {
                        if selected {
                            canvas.draw(&self.white_selected_square, pos);
                        }
                        else if moving {
                            canvas.draw(&self.white_moving_square, pos);
                        }
                        else if moved {
                            canvas.draw(&self.white_moved_square, pos);
                        }
                        else {
                            canvas.draw(&self.white_square, pos);
                        }
                        if moves[y][x] {
                            if your_turn(&self.turn, server_color, is_server) {
                                move_color = Some(self.move_rgb_white)
                            }
                            else {
                                move_color = Some(self.move_future_rgb_white)
                            }
                        }
/*                         if y == 7 {
                            let mut text = Text::new(TextFragment::new(cord_to_file(x as f32))
                                .color(self.white_rgb));
                            let _ = text.set_scale(CORD_FONT_SIZE);
                            
                            let text_pos = text_pos_y;
                            canvas.draw(&text, text_pos);
                        }
                        if x == 0 {
                            let mut text = Text::new(TextFragment::new(y.to_string())
                                .color(self.white_rgb));
                            let _ = text.set_scale(CORD_FONT_SIZE);
                            
                            let text_pos = text_pos_x;
                            canvas.draw(&text, text_pos);
                        } */
                    } 
                    else {
                        if selected {
                            canvas.draw(&self.black_selected_square, pos);
                        }
                        else if moving {
                            canvas.draw(&self.black_moving_square, pos);
                        }
                        else if moved {
                            canvas.draw(&self.black_moved_square, pos);
                        }
                        else {
                            canvas.draw(&self.black_square, pos);
                        }
/*                         if y == 7 {
                            let mut text = Text::new(TextFragment::new(cord_to_file(x as f32))
                                .color(self.black_rgb));
                            let _ = text.set_scale(CORD_FONT_SIZE);
                            
                            let text_pos = text_pos_y;
                            canvas.draw(&text, text_pos);
                        }
                        if x == 0 {
                            let mut text = Text::new(TextFragment::new(y.to_string())
                                .color(self.black_rgb));
                            let _ = text.set_scale(CORD_FONT_SIZE);
                            
                            let text_pos = text_pos_x;
                            canvas.draw(&text, text_pos);
                        } */
                        if moves[y][x] {
                            if your_turn(&self.turn, server_color, is_server) {
                                move_color = Some(self.move_rgb_black)
                            }
                            else {
                                move_color = Some(self.move_future_rgb_black)
                            }
                        }
                    }

                    if let Some(color) = move_color {
                        if self.board[y][x] == Piece::None {
                            canvas.draw(&self.move_circle, graphics::DrawParam::new()
                                .dest(pos + Vec2::new(SQUARE_SIZE / 2.0, SQUARE_SIZE / 2.0))
                                .color(color));
                        } else {
                            draw_captured_move(&mut canvas, pos, &color, &self.move_capture)
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

                    if self.selected == Some(pos_unit) && self.dragging && your_turn(&self.turn, server_color, is_server) && Some(your_color(server_color, is_server)) == piece_color(&self.board[y][x]) {
                        selected_image = Some(image);
                    }
                    else {
                        canvas.draw(image, graphics::DrawParam::new()
                            .dest(pos)
                            .scale(Vec2::new(0.75, 0.75)));
                    }
                }
            }

            if let Some(selected_image) = selected_image {
                canvas.draw(selected_image, graphics::DrawParam::new()
                    .dest(Vec2::new(self.pos_x - SQUARE_SIZE / 2.0, self.pos_y - SQUARE_SIZE / 2.0))
                    .scale(Vec2::new(0.75, 0.75)));
            }

            let mut controls_text = Text::new(&self.controls_text);
            controls_text.set_scale(PxScale::from(TEXT_SIZE));

            let _ = &self.text.set_scale(TEXT_SIZE)
                .set_bounds(Vec2::new(SIDEBAR_SIZE - TEXT_SIZE * 2.0, f32::INFINITY))
                .set_wrap(true);
            
            let _ = controls_text.set_bounds(Vec2::new(SIDEBAR_SIZE - TEXT_SIZE * 2.0, f32::INFINITY))
            .set_wrap(true);

            let controls_text_pos = Vec2::new(8.0 * SQUARE_SIZE + TEXT_SIZE, TEXT_SIZE);
            let text_pos = controls_text_pos + 
                Vec2::new( 0.0, controls_text.measure(ctx).unwrap().y + TEXT_SIZE);

            canvas.draw(&controls_text, controls_text_pos);
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
            .dest(Vec2::new(0.0, 0.0)));

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    ) -> GameResult {
        if (x - self.start_x).abs() + (y - self.start_y).abs() > DRAG_SENSITIVITY && ctx.mouse.button_pressed(MouseButton::Left) {
            self.dragging = true;
        }
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
        if self.tcp_started 
            && let Some(is_server) = self.is_server 
            && let Some(server_color) = &self.server_color 
            && x < 8.0 * SQUARE_SIZE 
            && y < 8.0 * SQUARE_SIZE 
            && x > 0.0
            && y > 0.0
            && button == MouseButton::Left 
            {
            let y_c = y_colored(is_server, server_color, (y as f32 / SQUARE_SIZE).floor() as usize);
            let x_c = x_colored(is_server, server_color, (x as f32 / SQUARE_SIZE).floor() as usize);

            let temp = Some(Vec2::new(x_c as f32, y_c as f32));
            
            if piece_color(&self.board[y_c as usize][x_c as usize]) != Some(your_color(server_color, is_server)) {
                return Ok(());
            }

            if self.joever != Joever::Ongoing 
                || self.selected == temp {
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
        self.dragging = false;

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
            let x_c = x_colored(is_server, server_color, (x as f32 / SQUARE_SIZE).floor() as usize);

            let mv = Move { 
                start_x: selected.x as usize, 
                start_y: selected.y as usize, 
                end_x: x_c, 
                end_y: y_c,
                promotion: Piece::None,
            };

            if !your_turn(&self.turn, server_color, is_server) 
                || Some(oposite_color(&your_color(server_color, is_server))) == piece_color(&self.board[selected.y as usize][selected.x as usize])
                || (mv.end_x == mv.start_x && mv.end_y == mv.start_y) {
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
                                self.board[move_made.end_y][move_made.end_x],
                                cords_to_square(move_made.start_x as f32, move_made.start_y as f32), 
                                cords_to_square(move_made.end_x as f32, move_made.end_y as f32)
                            ));
                        self.turn = turn;
                        self.moves = moves;
                        self.selected = None;
                        self.dragging = false;
                    },
                    TcpToGame::Error { message } => {
                        self.text = Text::new(
                            format!("Move error: {}", message));
                        self.selected = None;
                        self.dragging = false;
                    },
                    TcpToGame::Handshake { .. } => unreachable!(),
                }
            }
            self.selected = None;
        }
        Ok(())
    }
}

fn draw_captured_move(canvas: &mut graphics::Canvas, pos: Vec2, color: &graphics::Color, mesh: &graphics::Mesh) {
    canvas.draw(mesh, DrawParam::default()
        .dest(pos)
        .color(*color));

        canvas.draw(mesh, DrawParam {
            transform: graphics::Transform::Values {
                dest: ggez::mint::Point2::from(pos + Vec2::new(0.0, SQUARE_SIZE)),
                rotation: 3.0 * PI / 2.0,
                scale: ggez::mint::Vector2 { x: 1.0, y: 1.0 },
                offset: ggez::mint::Point2 { x: 0.0, y: 0.0 }
            },
            color: *color,
            .. Default::default()
        });

    canvas.draw(mesh, DrawParam {
        transform: graphics::Transform::Values {
            dest: ggez::mint::Point2::from(pos + Vec2::new(SQUARE_SIZE, 0.0)),
            rotation: PI / 2.0,
            scale: ggez::mint::Vector2 { x: 1.0, y: 1.0 },
            offset: ggez::mint::Point2 { x: 0.0, y: 0.0 }
        },
        color: *color,
        .. Default::default()
    });

    canvas.draw(mesh, DrawParam {
        transform: graphics::Transform::Values {
            dest: ggez::mint::Point2::from(pos + Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
            rotation: PI,
            scale: ggez::mint::Vector2 { x: 1.0, y: 1.0 },
            offset: ggez::mint::Point2 { x: 0.0, y: 0.0 }
        },
        color: *color,
        .. Default::default()
    });
}

fn your_color(server_color: &Color, is_server: bool) -> Color {
    if is_server {
        server_color.clone()
    }
    else {
        oposite_color(server_color)
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
    let y = y.clamp(0, 7);
    match server_color {
        chess_network_protocol::Color::White => 
        if !is_server { y } 
        else { 7 - y },
        chess_network_protocol::Color::Black => 
        if !is_server { 7 - y } 
        else { y },
    }
}

fn x_colored(is_server: bool, server_color: &chess_network_protocol::Color, x: usize) -> usize {
    let x = x.clamp(0, 7);
    match server_color {
        chess_network_protocol::Color::White => 
        if is_server { x } 
        else { 7 - x },
        chess_network_protocol::Color::Black => 
        if is_server { 7 - x } 
        else { x },
    }
}

fn cord_to_file(x: f32) -> String {
    match x as u8 {
        0 => "a".to_string(),
        1 => "b".to_string(),
        2 => "c".to_string(),
        3 => "d".to_string(),
        4 => "e".to_string(),
        5 => "f".to_string(),
        6 => "g".to_string(),
        7 => "h".to_string(),
        _ => unreachable!(),
    }
}

fn cords_to_square(x: f32, y: f32) -> String {
    cord_to_file(x) + (y + 1.0).to_string().as_str()
}

mod client;
mod server;

pub fn main() -> GameResult {
    std::env::set_var("RUST_BACKTRACE", "1");
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
