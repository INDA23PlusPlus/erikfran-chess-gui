use chess_network_protocol::*;

use redkar_chess as chess;

pub fn explain_move_error(e: chess::MoveError) -> String {
    match e {
        chess::MoveError::NoPiece => "There is no piece at the given position".to_string(),
        chess::MoveError::WrongColorPiece => "The piece at the given position is not the same color as the current player".to_string(),
        chess::MoveError::OutsideBoard => "The given position is outside the board".to_string(),
        chess::MoveError::FriendlyFire => "You can't capture your own pieces".to_string(),
        chess::MoveError::BlockedPath => "The path to the given position is blocked".to_string(),
        chess::MoveError::SelfCheck => "You can't put yourself in check".to_string(),
        chess::MoveError::Movement => "The piece can't move like that".to_string(),
        chess::MoveError::Mated => "You are in checkmate".to_string(),
    }
}

pub trait IntoNetwork<T> {
    fn into_network(self) -> T;
}

pub trait IntoChess<T> {
    fn into_chess(self) -> T;
}

pub trait IntoNetworkMove<T> {
    fn into_network_move(self, promotion: Piece) -> T;
}

impl IntoNetworkMove<Move> for chess::Move {
    fn into_network_move(self, promotion: Piece) -> Move {
        Move {
            start_x: self.start_x,
            start_y: self.start_y,
            end_x: self.end_x,
            end_y: self.end_y,
            promotion: promotion,
        }
    }
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

impl IntoNetwork<Color> for chess::Color {
    fn into_network(self) -> Color {
        match self {
            chess::Color::White => Color::White,
            chess::Color::Black => Color::Black,
        }
    }
}

impl IntoChess<chess::Color> for Color {
    fn into_chess(self) -> chess::Color {
        match self {
            Color::White => chess::Color::White,
            Color::Black => chess::Color::Black,
        }
    }
}

impl IntoNetwork<[[Piece; 8]; 8]> for [[Option<chess::Piece>; 8]; 8] {
    fn into_network(self) -> [[Piece; 8]; 8] {
        let mut new_board = [[Piece::None; 8]; 8];

        for (i, row) in self.iter().enumerate() {
            for (j, piece) in row.iter().enumerate() {
                new_board[i][j] = piece.into_network();
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

impl IntoNetwork<Piece> for Option<chess::Piece> {
    fn into_network(self) -> Piece {
        match self {
            Some(chess::Piece { piece: chess::PieceType::Bishop, color: chess::Color::Black }) => Piece::BlackBishop,
            Some(chess::Piece { piece: chess::PieceType::King, color: chess::Color::Black }) => Piece::BlackKing,
            Some(chess::Piece { piece: chess::PieceType::Knight, color: chess::Color::Black }) => Piece::BlackKnight,
            Some(chess::Piece { piece: chess::PieceType::Pawn, color: chess::Color::Black }) => Piece::BlackPawn,
            Some(chess::Piece { piece: chess::PieceType::Queen, color: chess::Color::Black }) => Piece::BlackQueen,
            Some(chess::Piece { piece: chess::PieceType::Rook, color: chess::Color::Black }) => Piece::BlackRook,
            Some(chess::Piece { piece: chess::PieceType::Bishop, color: chess::Color::White }) => Piece::WhiteBishop,
            Some(chess::Piece { piece: chess::PieceType::King, color: chess::Color::White }) => Piece::WhiteKing,
            Some(chess::Piece { piece: chess::PieceType::Knight, color: chess::Color::White }) => Piece::WhiteKnight,
            Some(chess::Piece { piece: chess::PieceType::Pawn, color: chess::Color::White }) => Piece::WhitePawn,
            Some(chess::Piece { piece: chess::PieceType::Queen, color: chess::Color::White }) => Piece::WhiteQueen,
            Some(chess::Piece { piece: chess::PieceType::Rook, color: chess::Color::White }) => Piece::WhiteRook,
            None => Piece::None,
        }
    }
}