use chess_network_protocol::*;

use redkar_chess as chess;

pub trait FromChess<T> {
    fn from_chess(t: T) -> Self;
}

impl FromChess<chess::Color> for Color {
    fn from_chess(color: chess::Color) -> Self {
        match color {
            chess::Color::White => Color::White,
            chess::Color::Black => Color::Black,
        }
    }
}

impl FromChess<Option<chess::Piece>> for Piece {
    fn from_chess(piece: Option<chess::Piece>) -> Self {
        match piece {
            Some(chess::Piece { piece: chess::PieceType::Pawn, color: chess::Color::Black }) => Piece::BlackPawn,
            Some(chess::Piece { piece: chess::PieceType::Pawn, color: chess::Color::White }) => Piece::WhitePawn,
            Some(chess::Piece { piece: chess::PieceType::Knight, color: chess::Color::Black }) => Piece::BlackKnight,
            Some(chess::Piece { piece: chess::PieceType::Knight, color: chess::Color::White }) => Piece::WhiteKnight,
            Some(chess::Piece { piece: chess::PieceType::Bishop, color: chess::Color::Black }) => Piece::BlackBishop,
            Some(chess::Piece { piece: chess::PieceType::Bishop, color: chess::Color::White }) => Piece::WhiteBishop,
            Some(chess::Piece { piece: chess::PieceType::Rook, color: chess::Color::Black }) => Piece::BlackRook,
            Some(chess::Piece { piece: chess::PieceType::Rook, color: chess::Color::White }) => Piece::WhiteRook,
            Some(chess::Piece { piece: chess::PieceType::Queen, color: chess::Color::Black }) => Piece::BlackQueen,
            Some(chess::Piece { piece: chess::PieceType::Queen, color: chess::Color::White }) => Piece::WhiteQueen,
            Some(chess::Piece { piece: chess::PieceType::King, color: chess::Color::Black }) => Piece::BlackKing,
            Some(chess::Piece { piece: chess::PieceType::King, color: chess::Color::White }) => Piece::WhiteKing,
            None => Piece::None,
        }
    }
}

trait FromChessMove {
    fn from_chess(m: chess::Move, p: Piece) -> Self;
}

impl FromChess<chess::Move> for Move {
    fn from_chess(m: chess::Move) -> Self {
        Move {
            start_x: m.start_x,
            start_y: m.start_y,
            end_x: m.end_x,
            end_y: m.end_y,
            promotion: Piece::None,
        }
    }
}

impl FromChessMove for Move {
    fn from_chess(m: chess::Move, p: Piece) -> Self {
        Move {
            start_x: m.start_x,
            start_y: m.start_y,
            end_x: m.end_x,
            end_y: m.end_y,
            promotion: p,
        }
    }
}

pub trait IntoChess<T> {
    fn into_chess(self) -> T;
}

impl IntoChess<chess::Move> for Move {
    fn into_chess(self) -> chess::Move {
        chess::Move {
            start_x: self.start_x,
            start_y: self.start_y,
            end_x: self.end_x,
            end_y: self.end_y,
        }
    }
}

pub trait IntoNetwork<T> {
    fn into_network(self) -> T;
}

impl IntoNetwork<[[Piece; 8]; 8]> for [[Option<chess::Piece>; 8]; 8] {
    fn into_network(self) -> [[Piece; 8]; 8] {
        let mut new_board = [[Piece::None; 8]; 8];

        for (i, row) in self.iter().enumerate() {
            for (j, piece) in row.iter().enumerate() {
                new_board[i][j] = Piece::from_chess(*piece);
            }
        }
        new_board
    }
}

impl FromChess<[[Option<chess::Piece>; 8]; 8]> for [[Piece; 8]; 8] {
    fn from_chess(board: [[Option<chess::Piece>; 8]; 8]) -> Self {
        let mut new_board = [[Piece::None; 8]; 8];

        for (i, row) in board.iter().enumerate() {
            for (j, piece) in row.iter().enumerate() {
                new_board[i][j] = Piece::from_chess(*piece);
            }
        }
        new_board
    }
}

impl IntoChess<[[Option<chess::Piece>; 8]; 8]> for [[Piece; 8]; 8] {
    fn into_chess(self) -> [[Option<chess::Piece>; 8]; 8] {
        let mut new_board = [[None; 8]; 8];

        for (i, row) in self.iter().enumerate() {
            for (j, piece) in row.iter().enumerate() {
                new_board[i][j] = piece.into_chess();
            }
        }
        new_board
    }
}

impl FromChess<Option<chess::Decision>> for Joever {
    fn from_chess(decision: Option<chess::Decision>) -> Self {
        match decision {
            Some(chess::Decision::Tie) => Joever::Draw,
            Some(chess::Decision::Black) => Joever::Black,
            Some(chess::Decision::White) => Joever::White,
            None => Joever::Ongoing,
        }
    }
}

impl IntoNetwork<Joever> for Option<chess::Decision> {
    fn into_network(self) -> Joever {
        match self {
            Some(chess::Decision::Tie) => Joever::Draw,
            Some(chess::Decision::Black) => Joever::Black,
            Some(chess::Decision::White) => Joever::White,
            None => Joever::Ongoing,
        }
    }
}

impl IntoChess<Option<chess::Decision>> for Joever {
    fn into_chess(self) -> Option<chess::Decision> {
        match self {
            Joever::Draw => Some(chess::Decision::Tie),
            Joever::Black => Some(chess::Decision::Black),
            Joever::White => Some(chess::Decision::White),
            Joever::Ongoing => None,
            Joever::Indeterminate => Some(chess::Decision::Tie),
        }
    }
}

impl IntoChess<Option<chess::Piece>> for Piece {
    fn into_chess(self) -> Option<chess::Piece> {
        match self {
            Self::BlackBishop => Some(chess::Piece { piece: chess::PieceType::Bishop, color: chess::Color::Black }),
            Self::BlackKing => Some(chess::Piece { piece: chess::PieceType::King, color: chess::Color::Black }),
            Self::BlackKnight => Some(chess::Piece { piece: chess::PieceType::Knight, color: chess::Color::Black }),
            Self::BlackPawn => Some(chess::Piece { piece: chess::PieceType::Pawn, color: chess::Color::Black }),
            Self::BlackQueen => Some(chess::Piece { piece: chess::PieceType::Queen, color: chess::Color::Black }),
            Self::BlackRook => Some(chess::Piece { piece: chess::PieceType::Rook, color: chess::Color::Black }),
            Self::WhiteBishop => Some(chess::Piece { piece: chess::PieceType::Bishop, color: chess::Color::White }),
            Self::WhiteKing => Some(chess::Piece { piece: chess::PieceType::King, color: chess::Color::White }),
            Self::WhiteKnight => Some(chess::Piece { piece: chess::PieceType::Knight, color: chess::Color::White }),
            Self::WhitePawn => Some(chess::Piece { piece: chess::PieceType::Pawn, color: chess::Color::White }),
            Self::WhiteQueen => Some(chess::Piece { piece: chess::PieceType::Queen, color: chess::Color::White }),
            Self::WhiteRook => Some(chess::Piece { piece: chess::PieceType::Rook, color: chess::Color::White }),
            Self::None => None,
        }
    }
}