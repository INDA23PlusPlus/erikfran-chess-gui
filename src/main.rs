use ggez::event::MouseButton;
use ggez::{event, conf};
use ggez::graphics::{self, Color, Rect};
use ggez::{Context, GameResult};
use ggez::glam::*;
use redkar_chess::*;
use core::fmt;
use std::fmt::Display;
use std::{env, path};

const SQUARE_SIZE: f32 = 130.0;

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
        };

        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            Color::BLACK,
        );

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
                let pos = Vec2::new(x as f32 * SQUARE_SIZE, y as f32 * SQUARE_SIZE);
                let pos_unit = Vec2::new(x as f32, y as f32);
                let selected = 
                    (self.selected == Some(pos_unit) 
                        && self.game.board[y][x].is_some())
                    || (pos_unit == last_move_pos_from 
                        || pos_unit == last_move_pos_to);
                
                if (x + y) % 2 == 0 {
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
                    canvas.draw(image, pos);
                }
            }
        }

        if self.selected.is_some() {
            canvas.draw(selected_image, relative_pos);
        }

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
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.start_x = x;
        self.start_y = y;
        self.selected = Some(Vec2::new((x / SQUARE_SIZE).floor(), (y / SQUARE_SIZE).floor()));
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
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
                    cords_to_square((x / SQUARE_SIZE).floor(), (y / SQUARE_SIZE).floor()));

                    self.last_move = Some(mv);

                    //TODO: add menu for handling game result
                    match result {
                        Some(Decision::White) => println!("White won!"),
                        Some(Decision::Black) => println!("Black won!"),
                        Some(Decision::Tie) => println!("Draw!"),
                        None => (),
                    }
                },
                Err(e) => println!("Move failed: {:?}", e),
            }
        }

        self.selected = None;
        Ok(())
    }
}

fn explain_move_error(e: MoveError) -> String {
    match e {
        MoveError::NoPiece => "There is no piece at the given position".to_string(),
        MoveError::WrongColorPiece => "The piece at the given position is not the same color as the current player".to_string(),
        MoveError::OutsideBoard => "The given position is outside the board".to_string(),
        MoveError::FriendlyFire => "You can't capture your own pieces".to_string(),
        MoveError::NoPiece => "There is no piece at the given position".to_string(),
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
        _ => panic!("Invalid x coordinate"),
    };

    t + match y as u8 {
        0 => "8".to_string(),
        1 => "7".to_string(),
        2 => "6".to_string(),
        3 => "5".to_string(),
        4 => "4".to_string(),
        5 => "3".to_string(),
        6 => "2".to_string(),
        7 => "1".to_string(),
        _ => panic!("Invalid y coordinate"),
    }.as_str()
}

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
            .dimensions(SQUARE_SIZE * 8.0 + 400.0, SQUARE_SIZE * 8.0),
    );
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state);
}
