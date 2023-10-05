use alvinw_chess::{game, piece::PieceType};
use chess_network_protocol::*;

use fritiofr_chess as chess;

use crate::server::UniversalGame;

pub struct Game {
    board: [[Piece; 8]; 8],
    turn: Color,
    joever: Joever,
    features: Vec<Features>,
    game: chess::Game,
}

impl UniversalGame for Game {
    fn new() -> Self {
        let game = chess::Game::start_pos();

        Self {
            board: game.get_board().into_network(),
            turn: Color::White,
            joever: Joever::Ongoing,
            features: vec![Features::Castling, Features::EnPassant, Features::Promotion, Features::PossibleMoveGeneration],
            game,
        }
    }

    fn try_move(&mut self, m: Move) -> Result<(), String> {
        let chess_moves = match self.game.gen_all_moves() {
            Some(m) => m,
            None => return Err("No moves available".to_string()),
        };

        let chess_move = match chess_moves.into_iter().find(|chess_move| chess_move.into_network() == m) {
            Some(m) => m,
            None => return Err("No such move exists".to_string()),
        };

        match self.game.apply_move(chess_move) {
            Ok(()) => {},
            Err(e) => return Err(format!("{e}")),
        }

        self.board = self.game.get_board().into_network();
        self.turn = self.game.get_turn().into_network();
        self.joever = self.game.is_checkmate().into_network(&self.turn);

        Ok(())
    }

    fn possible_moves(&self) -> Vec<Move> {
        match self.game.gen_all_moves() {
            Some(m) => m.into_iter().map(|m| m.into_network()).collect(),
            None => vec![],
        }
    }

    fn board(&self) -> [[Piece; 8]; 8] {
        self.board
    }

    fn turn(&self) -> Color {
        self.turn.clone()
    }

    fn joever(&self) -> Joever {
        self.joever
    }

    fn features(&self) -> Vec<Features> {
        self.features.clone()
    }
}

pub trait IntoNetwork<T> {
    fn into_network(self) -> T;
}

pub trait IntoChess<T> {
    fn into_chess(self) -> T;
}

pub trait IntoNetworkJoever {
    fn into_network(self, color: &Color) -> Joever;
}

impl IntoNetworkJoever for bool {
    fn into_network(self, color: &Color) -> Joever {
        match self {
            true => match color {
                Color::White => Joever::White,
                Color::Black => Joever::Black,
            },
            false => Joever::Ongoing,
        }
    }
}

/* pub trait IntoChessMoveGame<T> {
    fn into_chess_move_game(self, game: &Game) -> T;
}

impl IntoChessMoveGame<chess::Move> for Move {
    fn into_chess_move_game(self, game: &Game) -> chess::Move {
        let capture = game.board[self.end_y][self.end_x].into_chess();

        if game.board[self.start_y][self.start_x] == Piece::BlackKing || game.board[self.start_y][self.start_x] == Piece::WhiteKing {
            if self.start_x as i32 - self.end_x as i32 == 2 {
                return chess::Move::Castle { 
                    from: (self.start_x, self.start_y),
                    to: (self.end_x, self.end_y), 
                    rook_from: (0, self.start_y), 
                    rook_to: (3, self.start_y) 
                };
            }
            else if self.start_x as i32 - self.end_x as i32 == -2 {
                return chess::Move::Castle { 
                    from: (self.start_x, self.start_y),
                    to: (self.end_x, self.end_y), 
                    rook_from: (7, self.start_y), 
                    rook_to: (5, self.start_y) 
                };
            }
        }
        else if self.promotion != chess::PieceType::None {
            if capture.is_some() {
                return chess::Move::CapturePromotion { 
                    from: (self.start_x, self.start_y),
                    to: (self.end_x, self.end_y), 
                    capture: (self.end_x, self.end_y), 
                    promotion: self.promotion 
                };
            }
            return chess::Move::QuietPromotion { 
                from: (self.start_x, self.start_y),
                to: (self.end_x, self.end_y), 
                promotion: self.promotion 
            };
        }
        else if let Some(chess::Piece { piece_type: chess::PieceType::Pawn, .. }) = game.board.into_chess().get_tile(self.start_x, self.start_y) {
            if self.end_y == 0 || self.end_y == 7 {
                return chess::Move::QuietPromotion { 
                    from: (), 
                    to: (), 
                    promotion: self.promotion 
                } { 
                    from: (self.start_x, self.start_y),
                    to: (self.end_x, self.end_y), 
                    promotion: chess::PieceType::Queen 
                };
            }
        }

        chess::Move::Quiet { 
            from: (self.start_x, self.start_y), 
            to: (self.end_x, self.end_y) 
        }
    }
} */

impl IntoNetwork<Move> for chess::Move {
    fn into_network(self) -> Move {
        match self {
            chess::Move::Quiet { from, to } => Move {
                start_x: from.0,
                start_y: from.1,
                end_x: to.0,
                end_y: to.1,
                promotion: Piece::None,
            },
            chess::Move::Capture { from, to, .. } => Move {
                start_x: from.0,
                start_y: from.1,
                end_x: to.0,
                end_y: to.1,
                promotion: Piece::None,
            },
            chess::Move::QuietPromotion { from, to, promotion } => {
                let color = match to.1 {
                    0 => Color::Black,
                    7 => Color::White,
                    _ => unreachable!(),
                };

                Move {
                    start_x: from.0,
                    start_y: from.1,
                    end_x: to.0,
                    end_y: to.1,
                    promotion: promotion.into_network(&color),
                }
            },
            chess::Move::CapturePromotion { from, to, promotion, .. } => {
                let color = match to.1 {
                    0 => Color::Black,
                    7 => Color::White,
                    _ => unreachable!(),
                };

                Move {
                    start_x: from.0,
                    start_y: from.1,
                    end_x: to.0,
                    end_y: to.1,
                    promotion: promotion.into_network(&color),
                }
            },
            chess::Move::Castle { from, to, .. } => {
                Move {
                    start_x: from.0,
                    start_y: from.1,
                    end_x: to.0,
                    end_y: to.1,
                    promotion: Piece::None,
                }
            },
            chess::Move::DoublePawnPush { from, to } => {
                Move {
                    start_x: from.0,
                    start_y: from.1,
                    end_x: to.0,
                    end_y: to.1,
                    promotion: Piece::None,
                }
            },
        }
    }
}

impl IntoNetwork<Color> for chess::Color {
    fn into_network(self) -> Color {
        match self {
            chess::Color::White => Color::White,
            chess::Color::Black => Color::Black,
        }
    }
}

/* impl IntoChess<chess::PieceType> for Piece {
    fn into_chess(self) -> chess::PieceType {
        match self {
            Piece::BlackBishop => chess::PieceType::Bishop,
            Piece::BlackPawn => chess::PieceType::Pawn,
            Piece::BlackKing => chess::PieceType::King,
            Piece::BlackKnight => chess::PieceType::Knight,
            Piece::BlackPawn => chess::PieceType::Pawn,
            Piece::BlackQueen => chess::PieceType::Queen,
            Piece::BlackRook => chess::PieceType::Rook,
            Piece::WhiteBishop => chess::PieceType::Bishop,
            Piece::WhitePawn => chess::PieceType::Pawn,
            Piece::WhiteKing => chess::PieceType::King,
            Piece::WhiteKnight => chess::PieceType::Knight,
            Piece::WhitePawn => chess::PieceType::Pawn,
            Piece::WhiteQueen => chess::PieceType::Queen,
            Piece::WhiteRook => chess::PieceType::Rook,
            Piece::None => unreachable!(),
        }
    }
} */

pub trait IntoNetworkPiece {
    fn into_network(self, color: &Color) -> Piece;
}

impl IntoNetworkPiece for chess::PieceType {
    fn into_network(self, color: &Color) -> Piece {
        match color {
            Color::Black => match self {
                chess::PieceType::Bishop => Piece::BlackBishop,
                chess::PieceType::King => Piece::BlackKing,
                chess::PieceType::Knight => Piece::BlackKnight,
                chess::PieceType::Pawn => Piece::BlackPawn,
                chess::PieceType::Queen => Piece::BlackQueen,
                chess::PieceType::Rook => Piece::BlackRook,
            },
            Color::White => match self {
                chess::PieceType::Bishop => Piece::WhiteBishop,
                chess::PieceType::King => Piece::WhiteKing,
                chess::PieceType::Knight => Piece::WhiteKnight,
                chess::PieceType::Pawn => Piece::WhitePawn,
                chess::PieceType::Queen => Piece::WhiteQueen,
                chess::PieceType::Rook => Piece::WhiteRook,
            },
        }
    }
}

impl IntoNetwork<[[Piece; 8]; 8]> for chess::Board {
    fn into_network(self) -> [[Piece; 8]; 8] {
        let mut new_board = [[Piece::None; 8]; 8];

        for i in 0..8 {
            for j in 0..8 {
                new_board[i][j] = self.get_tile(j, i).into_network();
            }
        }
        new_board
    }
}

impl IntoChess<chess::Board> for [[Piece; 8]; 8] {
    fn into_chess(self) -> chess::Board {
        let mut new_board = Game::new().game.get_board();

        for (i, row) in self.iter().enumerate() {
            for (j, piece) in row.iter().enumerate() {
                match piece.into_chess() {
                    Some(p) => new_board.set_tile(j, i, p),
                    None => new_board.remove_tile(j, i),
                }
            }
        }

        new_board
    }
}

impl IntoNetwork<Piece> for Option<chess::Piece> {
    fn into_network(self) -> Piece {
        match self {
            Some(chess::Piece { piece_type: chess::PieceType::Bishop, color: chess::Color::Black }) => Piece::BlackBishop,
            Some(chess::Piece { piece_type: chess::PieceType::King, color: chess::Color::Black }) => Piece::BlackKing,
            Some(chess::Piece { piece_type: chess::PieceType::Knight, color: chess::Color::Black }) => Piece::BlackKnight,
            Some(chess::Piece { piece_type: chess::PieceType::Pawn, color: chess::Color::Black }) => Piece::BlackPawn,
            Some(chess::Piece { piece_type: chess::PieceType::Queen, color: chess::Color::Black }) => Piece::BlackQueen,
            Some(chess::Piece { piece_type: chess::PieceType::Rook, color: chess::Color::Black }) => Piece::BlackRook,
            Some(chess::Piece { piece_type: chess::PieceType::Bishop, color: chess::Color::White }) => Piece::WhiteBishop,
            Some(chess::Piece { piece_type: chess::PieceType::King, color: chess::Color::White }) => Piece::WhiteKing,
            Some(chess::Piece { piece_type: chess::PieceType::Knight, color: chess::Color::White }) => Piece::WhiteKnight,
            Some(chess::Piece { piece_type: chess::PieceType::Pawn, color: chess::Color::White }) => Piece::WhitePawn,
            Some(chess::Piece { piece_type: chess::PieceType::Queen, color: chess::Color::White }) => Piece::WhiteQueen,
            Some(chess::Piece { piece_type: chess::PieceType::Rook, color: chess::Color::White }) => Piece::WhiteRook,
            None => Piece::None,
        }
    }
}

impl IntoChess<Option<chess::Piece>> for Piece {
    fn into_chess(self) -> Option<chess::Piece> {
        match self {
            Piece::BlackBishop => Some(chess::Piece { piece_type: chess::PieceType::Bishop, color: chess::Color::Black }),
            Piece::BlackKing => Some(chess::Piece { piece_type: chess::PieceType::King, color: chess::Color::Black }),
            Piece::BlackKnight => Some(chess::Piece { piece_type: chess::PieceType::Knight, color: chess::Color::Black }),
            Piece::BlackPawn => Some(chess::Piece { piece_type: chess::PieceType::Pawn, color: chess::Color::Black }),
            Piece::BlackQueen => Some(chess::Piece { piece_type: chess::PieceType::Queen, color: chess::Color::Black }),
            Piece::BlackRook => Some(chess::Piece { piece_type: chess::PieceType::Rook, color: chess::Color::Black }),
            Piece::WhiteBishop => Some(chess::Piece { piece_type: chess::PieceType::Bishop, color: chess::Color::White }),
            Piece::WhiteKing => Some(chess::Piece { piece_type: chess::PieceType::King, color: chess::Color::White }),
            Piece::WhiteKnight => Some(chess::Piece { piece_type: chess::PieceType::Knight, color: chess::Color::White }),
            Piece::WhitePawn => Some(chess::Piece { piece_type: chess::PieceType::Pawn, color: chess::Color::White }),
            Piece::WhiteQueen => Some(chess::Piece { piece_type: chess::PieceType::Queen, color: chess::Color::White }),
            Piece::WhiteRook => Some(chess::Piece { piece_type: chess::PieceType::Rook, color: chess::Color::White }),
            Piece::None => None,
        }
    }
}