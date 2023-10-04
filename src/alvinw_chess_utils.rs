use chess::pos::BoardPos;
use chess_network_protocol::*;

use alvinw_chess as chess;

pub struct Game {
    pub board: [[Piece; 8]; 8],
    pub turn: Color,
    pub joever: Joever,
    pub features: Vec<Features>,
    game: chess::Game,
}

impl Game {
    pub fn new() -> Self {
        let game = chess::Game::new();

        Self {
            board: game.board.into_network(),
            turn: Color::White,
            joever: Joever::Ongoing,
            features: vec![Features::Castling, Features::PossibleMoveGeneration],
            game,
        }
    }

    pub fn try_move(&mut self, m: Move) -> Result<(), String> {
        match game.move_piece(from, to) {
            Ok(_) => {
                self.board = self.game.board.into_network();
                self.turn = self.game.turn.into_network();
                self.joever = self.game.joever.into_network();
                Ok(())
            },
            Err(MovePieceError::NoTile) => Err("The tile {from} is empty!"),
            Err(MovePieceError::NotCurrentTurn) => Err("You can not move your opponent's pieces!"),
            Err(MovePieceError::InvalidMove) => Err("That is not a valid move."),
        };
    }

    pub fn possible_moves(&self) -> Vec<Move> {
        vec![]
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

impl IntoNetworkMove<Move> for (chess::pos::BoardPos, chess::pos::BoardPos) {
    fn into_network_move(self, promotion: Piece) -> Move {
        Move {
            start_x: self.0.file() as usize,
            start_y: self.0.rank() as usize,
            end_x: self.1.file() as usize,
            end_y: self.1.rank() as usize,
            promotion: promotion,
        }
    }
}

impl IntoChess<(chess::pos::BoardPos, chess::pos::BoardPos)> for Move {
    fn into_chess(self) -> (chess::pos::BoardPos, chess::pos::BoardPos) {
        (BoardPos::new(self.start_x as u8, self.start_y as u8), BoardPos::new(self.end_x as u8, self.end_y as u8))
    }
}

impl IntoNetwork<Color> for chess::board::Color {
    fn into_network(self) -> Color {
        match self {
            chess::board::Color::White => Color::White,
            chess::board::Color::Black => Color::Black,
        }
    }
}

impl IntoChess<chess::board::Color> for Color {
    fn into_chess(self) -> chess::board::Color {
        match self {
            Color::White => chess::board::Color::White,
            Color::Black => chess::board::Color::Black,
        }
    }
}

impl IntoNetwork<[[Piece; 8]; 8]> for [[Option<chess::board::Tile>; 8]; 8] {
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

impl IntoChess<[[Option<chess::board::Tile>; 8]; 8]> for [[Piece; 8]; 8] {
    fn into_chess(self) -> [[Option<chess::board::Tile>; 8]; 8] {
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

impl IntoChess<Option<chess::board::Tile>> for Piece {
    fn into_chess(self) -> Option<chess::board::Tile> {
        match self {
            Self::BlackBishop => Some(chess::board::Tile { piece: chess::board::TileType::Bishop, color: chess::Color::Black }),
            Self::BlackKing => Some(chess::board::Tile { piece: chess::board::TileType::King, color: chess::Color::Black }),
            Self::BlackKnight => Some(chess::board::Tile { piece: chess::board::TileType::Knight, color: chess::Color::Black }),
            Self::BlackPawn => Some(chess::board::Tile { piece: chess::board::TileType::Pawn, color: chess::Color::Black }),
            Self::BlackQueen => Some(chess::board::Tile { piece: chess::board::TileType::Queen, color: chess::Color::Black }),
            Self::BlackRook => Some(chess::board::Tile { piece: chess::board::TileType::Rook, color: chess::Color::Black }),
            Self::WhiteBishop => Some(chess::board::Tile { piece: chess::board::TileType::Bishop, color: chess::Color::White }),
            Self::WhiteKing => Some(chess::board::Tile { piece: chess::board::TileType::King, color: chess::Color::White }),
            Self::WhiteKnight => Some(chess::board::Tile { piece: chess::board::TileType::Knight, color: chess::Color::White }),
            Self::WhitePawn => Some(chess::board::Tile { piece: chess::board::TileType::Pawn, color: chess::Color::White }),
            Self::WhiteQueen => Some(chess::board::Tile { piece: chess::board::TileType::Queen, color: chess::Color::White }),
            Self::WhiteRook => Some(chess::board::Tile { piece: chess::board::TileType::Rook, color: chess::Color::White }),
            Self::None => None,
        }
    }
}

impl IntoNetwork<Piece> for Option<chess::board::Tile> {
    fn into_network(self) -> Piece {
        let blackbishop = chess::board::Tile::new(chess::piece::PieceType::Bishop, chess::board::Color::Black);

        match self {
            Some() => Piece::BlackBishop,
            None => Piece::None,
        }
    }
}