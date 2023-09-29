use cgmath::Vector3;
use redkar_chess::*;
use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

use std::collections::HashMap;
use std::{env, path};

use std::iter;
use std::time::Instant;
use ggez::glam::Vec2;


use ::egui::FontDefinitions;
use egui_wgpu;
use erikfran_chess_gui::*;
use cgmath::Quaternion;

const INITIAL_WIDTH: u32 = SQUARE_SIZE as u32 * 8 + SIDEBAR_SIZE as u32;
const INITIAL_HEIGHT: u32 = SQUARE_SIZE as u32 * 8;

const SCALE: f32 = 0.75;
const SQUARE_SIZE: f32 = 130.0 * SCALE;
const TEXT_SIZE: f32 = 25.0 * SCALE;
const SIDEBAR_SIZE: f32 = 400.0;

struct MainState {
    game: Game,
    selected: Option<Vec2>,
    start_x: f32,
    start_y: f32,
    pos_x: f32,
    pos_y: f32,
    last_move: Option<Move>,
    decision: Option<Decision>,
    instances_hashmap: HashMap<String, Vec<Instance>>,
}

impl MainState {
    fn new() -> Self{
        let mut instances_hashmap: HashMap<String, Vec<Instance>> = HashMap::new();

        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Pawn, color: Color::White }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Rook, color: Color::White }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Knight, color: Color::White }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Bishop, color: Color::White }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Queen, color: Color::White }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::King, color: Color::White }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Pawn, color: Color::Black }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Rook, color: Color::Black }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Knight, color: Color::Black }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Bishop, color: Color::Black }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::Queen, color: Color::Black }), Vec::<Instance>::new());
        instances_hashmap.insert(piece_to_key(Piece { piece: PieceType::King, color: Color::Black }), Vec::<Instance>::new());

        MainState {
            game: Game::new_game(),
            selected: None,
            start_x: 0.0,
            start_y: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
            last_move: None,
            decision: None,
            instances_hashmap,
        }
    }
}

impl MainState {
    fn draw(&mut self) {
        let mut relative_pos: Vec2 = Vec2::new(0.0, 0.0);

        let (last_move_pos_from, last_move_pos_to) = match self.last_move {
            Some(mv) => (Vec2::new(mv.start_x as f32, mv.start_y as f32), Vec2::new(mv.end_x as f32, mv.end_y as f32)),
            None => (Vec2::new(-1.0, -1.0), Vec2::new(-1.0, -1.0)),
        };

        let mut selected_piece: Piece;

        for x in 0..8 {
            for y in 0..8 {
                let pos = Vec2::new(x as f32 * SQUARE_SIZE, y as f32 * SQUARE_SIZE);
                let pos_unit = Vec2::new(x as f32, y as f32);
                let selected = 
                    (self.selected == Some(pos_unit) 
                        && self.game.board[y][x].is_some())
                    || (pos_unit == last_move_pos_from 
                        || pos_unit == last_move_pos_to);
                
                if (x + y) % 2 == 0 {
                    if selected {
                        //canvas.draw(&black_selected_square, pos);
                    }
                    else {
                        //canvas.draw(&black_square, pos);
                    }
                } 
                else {
                    if selected {
                        //canvas.draw(&white_selected_square, pos);
                    }
                    else {
                        //canvas.draw(&white_square, pos);
                    }
                }

                let piece = match self.game.board[y][x] {
                    Some(piece) => piece,
                    None => continue,
                };

                if self.selected == Some(pos_unit) && self.game.board[y][x].is_some() {
                    relative_pos = pos + Vec2::new(self.pos_x - self.start_x, self.pos_y - self.start_y);
                    selected_piece = piece;
                }
                else {
                    self.instances_hashmap[&piece_to_key(piece)].push(
                        Instance { 
                            position: Vector3::new(pos.x, pos.y, 0.0),
                            rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0),
                            scale: MODEL_SCALE,
                        });
                }
            }
        }

        if self.selected.is_some() {
            self.instances_hashmap[&piece_to_key(selected_piece)].push(
                Instance { 
                    position: Vector3::new(relative_pos.x, relative_pos.y, 0.0),
                    rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0),
                    scale: MODEL_SCALE,
                });
        }

        if let Some(r) = &self.decision {
            match r {
                Decision::Black => {
                    println!("Black won!");
                },
                Decision::White => {
                    println!("White won!");
                },
                Decision::Tie => {
                    println!("Draw!");
                },
            }
        }
    }

    fn mouse_motion_event(
        &mut self,
        x: f32,
        y: f32,
    ) {

        // Mouse coordinates are PHYSICAL coordinates, but here we want logical coordinates.

        // If you simply use the initial coordinate system, then physical and logical
        // coordinates are identical.
        self.pos_x = x;
        self.pos_y = y;

        // If you change your screen coordinate system you need to calculate the
        // logical coordinates like this:
        /*
        let screen_rect = graphics::screen_coordinates(_ctx);
        let size = graphics::window(_ctx).inner_size();
        self.pos_x = (x / (size.width  as f32)) * screen_rect.w + screen_rect.x;
        self.pos_y = (y / (size.height as f32)) * screen_rect.h + screen_rect.y;
        */

        //println!("Mouse motion, x: {x}, y: {y}, relative x: {xrel}, relative y: {yrel}");
    }

    fn mouse_button_down_event(
        &mut self,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if let MouseButton::Left = button {
            if self.decision.is_none() {
                self.start_x = x;
                self.start_y = y;
                self.selected = Some(Vec2::new((x / SQUARE_SIZE).floor(), (y / SQUARE_SIZE).floor()));
            }
        }
    }

    fn mouse_button_up_event(
        &mut self,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if let MouseButton::Left = button {
            if self.decision.is_some() {
                return;
            }

            let mv = Move { 
                start_x: self.selected.unwrap().x as usize, 
                start_y: self.selected.unwrap().y as usize, 
                end_x: (x / SQUARE_SIZE).floor() as usize, 
                end_y: (y / SQUARE_SIZE).floor() as usize
            };

            if self.selected != Some(Vec2::new((x / SQUARE_SIZE).floor(), (y / SQUARE_SIZE).floor())) {
                match self.game.do_move(mv) {
                    Ok(result) => {
                        println!("{:?} moved {:?} from {} to {}",
                            self.game.turn, 
                            self.game.board[self.selected.unwrap().y as usize][self.selected.unwrap().x as usize],
                            cords_to_square(self.selected.unwrap().x, self.selected.unwrap().y), 
                            cords_to_square((x / SQUARE_SIZE).floor(), (y / SQUARE_SIZE).floor())
                        );

                        self.last_move = Some(mv);

                        self.decision = result;
                    },
                    Err(e) => println!("Move error: {}", explain_move_error(e)),
                }
            }

            self.selected = None;
        }
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


pub fn run() {
    let mut main_state = MainState::new();

    env_logger::init();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

	let mut state = State::new(window).await;

    // Handle events. Refer to `winit` docs for more information.
	event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
				ref event,
				window_id,
			} if window_id == state.window().id() => if !state.input(event) {
				match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        main_state.mouse_motion_event(ctx, position.x, position.y);
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        match state {
                            winit::event::ElementState::Pressed => {
                                main_state.mouse_button_down_event(ctx, *button, main_state.mouse_position_x, main_state.mouse_position_y);
                            },
                            winit::event::ElementState::Released => {
                                main_state.mouse_button_up_event(ctx, *button, main_state.mouse_position_x, main_state.mouse_position_y);
                            },
                        }
                    },
					WindowEvent::CloseRequested
					| WindowEvent::KeyboardInput {
						input:
							KeyboardInput {
								state: ElementState::Pressed,
								virtual_keycode: Some(VirtualKeyCode::Escape),
								..
							},
						..
					} => {
                        *control_flow = ControlFlow::Exit
                    },
					WindowEvent::Resized(physical_size) => {
						state.resize(*physical_size);
					}
					WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
						// new_inner_size is &&mut so we have to dereference it twice
						state.resize(**new_inner_size);
					}
					_ => {}
				}
			}
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
				// request it.
				state.window().request_redraw();
            },
			Event::RedrawRequested(window_id) if window_id == state.window().id() => {
				state.update();
                for i in 0..main_state.instances_hashmap.len() {
                    match state.render(
                        main_state.instances_hashmap.value()[i], 
                        key_to_piece(main_state.instances_hashmap.keys()[i])
                    ) {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
			},
			_ => *control_flow = ControlFlow::Poll
        }
    });
    Ok(())
}

const MODEL_SCALE: f32 = 1.0;