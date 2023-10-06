use chess_network_protocol::*;

use chess as chess;

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
        let game = chess::Game::new();

        Self {
            board: game.board.into_network(),
            turn: Color::White,
            joever: Joever::Ongoing,
            features: vec![Features::Castling, Features::PossibleMoveGeneration],
            game,
        }
    }

    fn try_move(&mut self, mv: Move) -> Result<(), String> {
        let mut moves = vec![];

        for y in 0..8 {
            for x in 0..8 {
                if let Ok((m, castles)) = self.game.possible_moves(chess::util::Square::try_from((x, y)).unwrap(), true){
                    for s in chess::util::get_square_array() {
                        if let Some(mv) = m[s] {
                            moves.push((mv, y as usize));
                        }
                    }
    
                    for c in castles {
                        moves.push((c, y as usize));
                    }
                }
            }
        }

        let (chess_move, _) = match moves.into_iter().find(|(chess_move, rank)| chess_move.into_network(*rank) == mv) {
            Some(m) => m,
            None => return Err("That move is not one of the generated possible moves".to_string()),
        };

        match self.game.try_move(chess_move) {
            Ok(()) => {},
            Err(e) => return Err(format!("{e}")),
        }

        self.board = self.game.board.into_network();
        self.turn = self.game.turn.into_network();
        self.joever = self.game.game_status.into_network();

        Ok(())
    }

    fn possible_moves(&mut self) -> Vec<Move> {
        let mut moves = vec![];

        for y in 0..8 {
            for x in 0..8 {
                if let Ok((m, castles)) = self.game.possible_moves(chess::util::Square::try_from((x, y)).unwrap(), true){
                    for s in chess::util::get_square_array() {
                        if let Some(mv) = m[s] {
                            moves.push(mv.into_network(y as usize));
                        }
                    }
    
                    for c in castles {
                        moves.push(c.into_network(y as usize));
                    }
                }
                
            }
        }

        moves
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

pub trait IntoNetworkMove<T> {
    fn into_network(self, rank: usize) -> T;
}

impl IntoNetwork<Joever> for chess::GameStatus {
    fn into_network(self) -> Joever {
        match self {
            chess::GameStatus::Ongoing => Joever::Ongoing,
            chess::GameStatus::Promoting => Joever::Ongoing,
            chess::GameStatus::Checkmate(c) => match c {
                chess::Color::White => Joever::Black,
                chess::Color::Black => Joever::White,
            },
        }
    }
}

impl IntoNetworkMove<Move> for chess::Move {
    fn into_network(self, rank: usize) -> Move {
        match self {
            chess::Move::Normal { from, to } => Move {
                start_x: i32::from(from.file) as usize,
                start_y: i32::from(from.rank) as usize,
                end_x: i32::from(to.file) as usize,
                end_y: i32::from(to.rank) as usize,
                promotion: Piece::None,
            },
            chess::Move::Castle { side } => {
                match side {
                    chess::CastlingSide::KingSide => Move {
                        start_x: 4,
                        start_y: rank,
                        end_x: 6,
                        end_y: rank,
                        promotion: Piece::None,
                    },
                    chess::CastlingSide::QueenSide => Move {
                        start_x: 4,
                        start_y: rank,
                        end_x: 2,
                        end_y: rank,
                        promotion: Piece::None,
                    },
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

/* pub trait IntoNetworkPiece {
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
} */

impl IntoNetwork<[[Piece; 8]; 8]> for chess::util::Board {
    fn into_network(self) -> [[Piece; 8]; 8] {
        let mut new_board = [[Piece::None; 8]; 8];

        for (y, rank) in chess::util::RANK_ARRAY.iter().enumerate() {
            for (x, file) in chess::util::FILE_ARRAY.iter().enumerate() {
                new_board[y][x] = self[*rank][*file].into_network();
            }
        }
        println!("{:?}", new_board);
        new_board
    }
}

/* impl IntoChess<chess::Board> for [[Piece; 8]; 8] {
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
} */

impl IntoNetwork<Piece> for Option<chess::Piece> {
    fn into_network(self) -> Piece {
        match self {
            Some(chess::Piece { piece: chess::PieceTypes::Bishop, color: chess::Color::Black }) => Piece::BlackBishop,
            Some(chess::Piece { piece: chess::PieceTypes::King, color: chess::Color::Black }) => Piece::BlackKing,
            Some(chess::Piece { piece: chess::PieceTypes::Knight, color: chess::Color::Black }) => Piece::BlackKnight,
            Some(chess::Piece { piece: chess::PieceTypes::Pawn(_), color: chess::Color::Black }) => Piece::BlackPawn,
            Some(chess::Piece { piece: chess::PieceTypes::Queen, color: chess::Color::Black }) => Piece::BlackQueen,
            Some(chess::Piece { piece: chess::PieceTypes::Rook, color: chess::Color::Black }) => Piece::BlackRook,
            Some(chess::Piece { piece: chess::PieceTypes::Bishop, color: chess::Color::White }) => Piece::WhiteBishop,
            Some(chess::Piece { piece: chess::PieceTypes::King, color: chess::Color::White }) => Piece::WhiteKing,
            Some(chess::Piece { piece: chess::PieceTypes::Knight, color: chess::Color::White }) => Piece::WhiteKnight,
            Some(chess::Piece { piece: chess::PieceTypes::Pawn(_), color: chess::Color::White }) => Piece::WhitePawn,
            Some(chess::Piece { piece: chess::PieceTypes::Queen, color: chess::Color::White }) => Piece::WhiteQueen,
            Some(chess::Piece { piece: chess::PieceTypes::Rook, color: chess::Color::White }) => Piece::WhiteRook,
            None => Piece::None,
        }
    }
}

/* impl IntoChess<Option<chess::Piece>> for Piece {
    fn into_chess(self) -> Option<chess::Piece> {
        match self {
            Piece::BlackBishop => Some(chess::Piece { piece: chess::PieceTypes::Bishop, color: chess::Color::Black }),
            Piece::BlackKing => Some(chess::Piece { piece: chess::PieceTypes::King, color: chess::Color::Black }),
            Piece::BlackKnight => Some(chess::Piece { piece: chess::PieceTypes::Knight, color: chess::Color::Black }),
            Piece::BlackPawn => Some(chess::Piece { piece: chess::PieceTypes::Pawn, color: chess::Color::Black }),
            Piece::BlackQueen => Some(chess::Piece { piece: chess::PieceTypes::Queen, color: chess::Color::Black }),
            Piece::BlackRook => Some(chess::Piece { piece: chess::PieceTypes::Rook, color: chess::Color::Black }),
            Piece::WhiteBishop => Some(chess::Piece { piece: chess::PieceTypes::Bishop, color: chess::Color::White }),
            Piece::WhiteKing => Some(chess::Piece { piece: chess::PieceTypes::King, color: chess::Color::White }),
            Piece::WhiteKnight => Some(chess::Piece { piece: chess::PieceTypes::Knight, color: chess::Color::White }),
            Piece::WhitePawn => Some(chess::Piece { piece: chess::PieceTypes::Pawn, color: chess::Color::White }),
            Piece::WhiteQueen => Some(chess::Piece { piece: chess::PieceTypes::Queen, color: chess::Color::White }),
            Piece::WhiteRook => Some(chess::Piece { piece: chess::PieceTypes::Rook, color: chess::Color::White }),
            Piece::None => None,
        }
    }
} */