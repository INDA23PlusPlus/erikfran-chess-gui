#![feature(let_chains)]
use ggez::event::MouseButton;
use ggez::mint::Vector2;
use ggez::{event, conf};
use ggez::graphics::{self, Color, Rect, Text, TextFragment, PxScale, DrawParam};
use ggez::{Context, GameResult, glam};
use ggez::glam::*;

use chess_network_protocol;

use redkar_chess::*;

use core::fmt;
use std::fmt::Display;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::{env, path, thread};

use ggegui::{egui, Gui};
use ggegui::egui::{TextStyle::*, FontId, FontFamily::Proportional};

const SCALE: f32 = 0.75;
const SQUARE_SIZE: f32 = 130.0 * SCALE;
const TEXT_SIZE: f32 = 25.0 * SCALE;
const SIDEBAR_SIZE: f32 = 400.0;
const UI_SCALE: f32 = 10.0;
const FONT_SIZE: f32 = 32.0;

fn chess_move_to_move(chess_move: redkar_chess::Move) -> chess_network_protocol::Move {
    chess_network_protocol::Move {
        start_x: chess_move.start_x,
        start_y: chess_move.start_y,
        end_x: chess_move.end_x,
        end_y: chess_move.end_y,
        promotion: chess_network_protocol::Piece::None,
    }
}

fn network_move_to_move(chess_move: chess_network_protocol::Move) -> Move {
    Move {
        start_x: chess_move.start_x,
        start_y: chess_move.start_y,
        end_x: chess_move.end_x,
        end_y: chess_move.end_y,
    }
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
    game: Game,
    selected: Option<Vec2>,
    start_x: f32,
    start_y: f32,
    pos_x: f32,
    pos_y: f32,
    last_move: Option<Move>,
    controls_text: graphics::Text,
    text: Text,
    decision: Option<Decision>,
    gui: Gui,
    is_server: Option<bool>,
    server_color: Option<chess_network_protocol::Color>,
    tcp_started: bool,
    client_state: Option<client::ClientToGame>,
    server_state: Option<server::ServerToGame>,
    client_receiver: Option<Receiver<client::ClientToGame>>,
    client_sender: Option<Sender<client::GameToClient>>,
    server_receiver: Option<Receiver<server::ServerToGame>>,
    server_sender: Option<Sender<server::GameToServer>>,
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
            game: Game::new_game(),
            selected: None,
            start_x: 0.0,
            start_y: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
            last_move: None,
            controls_text,
            text: Text::new(""),
            decision: None,
            gui,
            is_server: None,
            server_color: None,
            tcp_started: false,
            client_state: None,
            server_state: None,
            client_receiver: None,
            client_sender: None,
            server_receiver: None,
            server_sender: None,
        };

        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
	fn update(&mut self, ctx: &mut Context) -> GameResult {
        let gui_ctx = self.gui.ctx();

        if let Some(is_server) = self.is_server {
            if !is_server {
                if !self.tcp_started {
                    if let Some(server_color) = self.server_color {
                        self.tcp_started = true;
                        
                        let (client_sender, client_receiver) = std::sync::mpsc::channel();
                        let (game_sender, game_receiver) = std::sync::mpsc::channel();
                        
                        self.client_receiver = Some(client_receiver);
                        self.client_sender = Some(game_sender);

                        thread::spawn(move || client::run(client_sender, game_receiver, server_color));
                    }
                }
                else {
                    if let Ok(state) = self.client_receiver.as_mut().unwrap().try_recv() {
                        self.client_state = Some(state);
                    }
                }
                if self.server_color.is_none() {
                    egui::Area::new("").show(&gui_ctx, |ui| {
                        //gui_ctx.set_pixels_per_point(UI_SCALE);
                        ui.label("Want color do you want to play as?");
                        if ui.button("White").clicked() {
                            self.server_color = Some(chess_network_protocol::Color::Black);
                        }
                        if ui.button("Black").clicked() {
                            self.server_color = Some(chess_network_protocol::Color::White);
                        }
                    });
                }
            }
            else {
                if !self.tcp_started {
                    self.tcp_started = true;
                    
                    let (server_sender, server_receiver) = std::sync::mpsc::channel();
                    let (game_sender, game_receiver) = std::sync::mpsc::channel();
                    
                    self.server_receiver = Some(server_receiver);
                    self.server_sender = Some(game_sender);

                    thread::spawn(move || server::run(server_sender, game_receiver));
                }
                else {
                    if self.server_color.is_none() {
                        egui::Area::new("").show(&gui_ctx, |ui| {
                            //gui_ctx.set_pixels_per_point(UI_SCALE);
                            ui.label("Waiting for client to connect...");
                        });
                    }
                    if let Ok(state) = self.server_receiver.as_mut().unwrap().try_recv() {
                        self.server_color = Some(state.server_color.clone());
                        self.last_move = Some(network_move_to_move(state.move_made.clone()));
                        self.game = state.game.clone();
                    }
                }
            }
        }
        else {
            egui::Area::new("").show(&gui_ctx, |ui| {
                //gui_ctx.set_pixels_per_point(UI_SCALE);
                ui.label("Want to start a session as server or client?");
                if ui.button("Server").clicked() {
                    self.is_server = Some(true);
                }
                if ui.button("Client").clicked() {
                    self.is_server = Some(false);
                }
            });
        }

		self.gui.update(ctx);
		Ok(())
	}

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            Color::BLACK,
        );

        if let Some(is_server) = self.is_server && let Some(server_color) = &self.server_color {

            let white_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                Color::WHITE,
            )?;
            let black_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                Color::from_rgb(180, 135, 103,),
            )?;
            let white_selected_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                Color::from_rgb(207, 209, 134)
            )?;
            let black_selected_square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
                Color::from_rgb(170, 162, 87)
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
                            && self.game.board[y][x].is_some())
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

                    let image = match self.game.board[y][x] {
                        Some( Piece { piece: PieceType::Pawn, color: redkar_chess::Color::White } ) => &self.pawn_image_w,
                        Some( Piece { piece: PieceType::Pawn, color: redkar_chess::Color::Black } ) => &self.pawn_image_b,
                        Some( Piece { piece: PieceType::King, color: redkar_chess::Color::White } ) => &self.king_image_w,
                        Some( Piece { piece: PieceType::King, color: redkar_chess::Color::Black } ) => &self.king_image_b,
                        Some( Piece { piece: PieceType::Queen, color: redkar_chess::Color::White } ) => &self.queen_image_w,
                        Some( Piece { piece: PieceType::Queen, color: redkar_chess::Color::Black } ) => &self.queen_image_b,
                        Some( Piece { piece: PieceType::Bishop, color: redkar_chess::Color::White } ) => &self.bishop_image_w,
                        Some( Piece { piece: PieceType::Bishop, color: redkar_chess::Color::Black } ) => &self.bishop_image_b,
                        Some( Piece { piece: PieceType::Knight, color: redkar_chess::Color::White } ) => &self.knight_image_w,
                        Some( Piece { piece: PieceType::Knight, color: redkar_chess::Color::Black } ) => &self.knight_image_b,
                        Some( Piece { piece: PieceType::Rook, color: redkar_chess::Color::White } ) => &self.rook_image_w,
                        Some( Piece { piece: PieceType::Rook, color: redkar_chess::Color::Black } ) => &self.rook_image_b,
                        None => continue,
                    };

                    if self.selected == Some(pos_unit) && self.game.board[y][x].is_some() {
                        relative_pos = pos + Vec2::new(self.pos_x - self.start_x, self.pos_y - self.start_y);
                        selected_image = image;
                    }
                    else {
                        canvas.draw(image, graphics::DrawParam::new()
                            .dest(pos)
                            .scale(Vec2::new(0.75, 0.75)));
                    }
                }
            }

            if self.selected.is_some() {
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

            if let Some(r) = &self.decision {
                let mut text = Text::new("");
                match r {
                    Decision::Black => {
                        text = Text::new("Black won!");
                    },
                    Decision::White => {
                        text = Text::new("White won!");
                    },
                    Decision::Tie => {
                        text = Text::new("Draw!");
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
        if self.decision.is_none() {
            if let Some(is_server) = self.is_server && let Some(server_color) = &self.server_color {
                let y_c = y_colored(is_server, server_color, (y as f32 / SQUARE_SIZE).floor() as usize);
                self.start_x = x;
                self.start_y = y;
                self.selected = Some(Vec2::new((x / SQUARE_SIZE).floor(), y_c as f32));
            }
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if self.decision.is_some() {
            return Ok(());
        }

        if let Some(is_server) = self.is_server && let Some(server_color) = &self.server_color {
            let y_c = y_colored(is_server, server_color, (y as f32 / SQUARE_SIZE).floor() as usize);

            let mv = Move { 
                start_x: self.selected.unwrap().x as usize, 
                start_y: self.selected.unwrap().y as usize, 
                end_x: (x / SQUARE_SIZE).floor() as usize, 
                end_y: (y / SQUARE_SIZE).floor() as usize
            };

            if self.selected != Some(Vec2::new((x / SQUARE_SIZE).floor(), y_c as f32)) {
                if let Some(sender) = &self.client_sender {
                    sender.send(client::GameToClient {
                        move_made: chess_move_to_move(mv),
                    }).unwrap();
                }
                if let Some(sender) = &self.server_sender {
                    sender.send(server::GameToServer {
                        move_made: chess_move_to_move(mv),
                    }).unwrap();
                }
                /*match self.game.do_move(mv) {
                    Ok(result) => {
                        self.text = Text::new(
                            format!("{:?} moved {:?} from {} to {}",
                                self.game.turn, 
                                self.game.board[self.selected.unwrap().y as usize][self.selected.unwrap().x as usize],
                                cords_to_square(self.selected.unwrap().x, self.selected.unwrap().y), 
                                cords_to_square((x / SQUARE_SIZE).floor(), (y / SQUARE_SIZE).floor())
                            ));

                        self.last_move = Some(mv);

                        self.decision = result;
                    },
                    Err(e) => self.text = Text::new(
                        format!("Move error: {}", explain_move_error(e)))
                }*/
            }

            self.selected = None;
        }

        Ok(())
    }
}

fn explain_move_error(e: MoveError) -> String {
    match e {
        MoveError::NoPiece => "There is no piece at the given position".to_string(),
        MoveError::WrongColorPiece => "The piece at the given position is not the same color as the current player".to_string(),
        MoveError::OutsideBoard => "The given position is outside the board".to_string(),
        MoveError::FriendlyFire => "You can't capture your own pieces".to_string(),
        MoveError::BlockedPath => "The path to the given position is blocked".to_string(),
        MoveError::SelfCheck => "You can't put yourself in check".to_string(),
        MoveError::Movement => "The piece can't move like that".to_string(),
        MoveError::Mated => "You are in checkmate".to_string(),
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
