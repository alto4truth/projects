use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

pub const BOARD_SQUARES: usize = 64;
pub const DEFAULT_POLICY_WIDTH: usize = 8;
pub const TINY_INPUT_SIZE: usize = 96;
pub const TINY_HIDDEN_SIZE: usize = 32;
pub const TINY_MOVE_FEATURES: usize = 12;
const PROMOTION_PIECES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub fn fen_symbol(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    pub fn from_fen_symbol(symbol: char) -> Option<Self> {
        match symbol {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub fn centipawn_value(self) -> f32 {
        match self {
            PieceKind::Pawn => 100.0,
            PieceKind::Knight => 320.0,
            PieceKind::Bishop => 330.0,
            PieceKind::Rook => 500.0,
            PieceKind::Queen => 900.0,
            PieceKind::King => 20_000.0,
        }
    }

    pub fn piece_square_bonus(self, square: Square, color: Color) -> f32 {
        let rank = if color == Color::White {
            square.rank() as f32
        } else {
            7.0 - square.rank() as f32
        };
        let file = square.file() as f32;
        let center_distance = (file - 3.5).abs() + (rank - 3.5).abs();
        match self {
            PieceKind::Pawn => rank * 6.0 - (file - 3.5).abs() * 1.5,
            PieceKind::Knight => 18.0 - center_distance * 4.0,
            PieceKind::Bishop => 14.0 - center_distance * 2.5 + rank * 1.5,
            PieceKind::Rook => rank * 3.0 - (file - 3.5).abs(),
            PieceKind::Queen => 8.0 - center_distance * 1.5,
            PieceKind::King => {
                let shelter = if rank <= 1.0 { 18.0 } else { 0.0 };
                shelter - center_distance * 2.0
            }
        }
    }

    pub fn san_symbol(self) -> Option<char> {
        match self {
            PieceKind::Pawn => None,
            PieceKind::Knight => Some('N'),
            PieceKind::Bishop => Some('B'),
            PieceKind::Rook => Some('R'),
            PieceKind::Queen => Some('Q'),
            PieceKind::King => Some('K'),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub fn fen_symbol(self) -> char {
        let symbol = match self.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };
        match self.color {
            Color::White => symbol.to_ascii_uppercase(),
            Color::Black => symbol,
        }
    }

    pub fn from_fen_symbol(symbol: char) -> Option<Self> {
        let color = if symbol.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let kind = match symbol.to_ascii_lowercase() {
            'p' => PieceKind::Pawn,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'r' => PieceKind::Rook,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            _ => return None,
        };
        Some(Self { color, kind })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub u8);

impl Square {
    pub fn new(index: u8) -> Option<Self> {
        if index < BOARD_SQUARES as u8 {
            Some(Self(index))
        } else {
            None
        }
    }

    pub fn from_file_rank(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self(rank * 8 + file))
        } else {
            None
        }
    }

    pub fn file(self) -> u8 {
        self.0 % 8
    }

    pub fn rank(self) -> u8 {
        self.0 / 8
    }

    pub fn from_algebraic(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let file = bytes[0].to_ascii_lowercase();
        let rank = bytes[1];
        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return None;
        }
        Self::from_file_rank(file - b'a', rank - b'1')
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        write!(f, "{file}{rank}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChessMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl ChessMove {
    pub fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }

    pub fn with_promotion(mut self, promotion: PieceKind) -> Self {
        self.promotion = Some(promotion);
        self
    }

    pub fn uci(&self) -> String {
        let mut uci = format!("{}{}", self.from, self.to);
        if let Some(promotion) = self.promotion {
            uci.push(match promotion {
                PieceKind::Knight => 'n',
                PieceKind::Bishop => 'b',
                PieceKind::Rook => 'r',
                PieceKind::Queen => 'q',
                PieceKind::Pawn | PieceKind::King => 'q',
            });
        }
        uci
    }

    pub fn from_uci(uci: &str) -> Option<Self> {
        if uci.len() < 4 {
            return None;
        }
        let from = Square::from_algebraic(&uci[0..2])?;
        let to = Square::from_algebraic(&uci[2..4])?;
        let promotion =
            uci.as_bytes()
                .get(4)
                .and_then(|symbol| match symbol.to_ascii_lowercase() {
                    b'n' => Some(PieceKind::Knight),
                    b'b' => Some(PieceKind::Bishop),
                    b'r' => Some(PieceKind::Rook),
                    b'q' => Some(PieceKind::Queen),
                    _ => None,
                });
        Some(Self {
            from,
            to,
            promotion,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEntry {
    pub chess_move: ChessMove,
    pub prior: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyValue {
    pub value: f32,
    pub policy: Vec<PolicyEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn from_fen(field: &str) -> Result<Self, String> {
        if field == "-" {
            return Ok(Self::default());
        }
        let mut rights = Self::default();
        for symbol in field.chars() {
            match symbol {
                'K' => rights.white_kingside = true,
                'Q' => rights.white_queenside = true,
                'k' => rights.black_kingside = true,
                'q' => rights.black_queenside = true,
                _ => return Err("invalid castling rights in FEN".to_string()),
            }
        }
        Ok(rights)
    }

    pub fn to_fen(self) -> String {
        let mut field = String::new();
        if self.white_kingside {
            field.push('K');
        }
        if self.white_queenside {
            field.push('Q');
        }
        if self.black_kingside {
            field.push('k');
        }
        if self.black_queenside {
            field.push('q');
        }
        if field.is_empty() {
            field.push('-');
        }
        field
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Checkmate { loser: Color },
    Stalemate { side_to_move: Color },
    DrawByFiftyMoveRule,
    DrawByRepetition,
}

#[derive(Debug, Clone)]
pub struct Position {
    board: [Option<Piece>; BOARD_SQUARES],
    side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant_target: Option<Square>,
    halfmove_clock: u16,
    fullmove_number: u16,
}

impl Default for Position {
    fn default() -> Self {
        Self::starting_position()
    }
}

impl Position {
    pub const STARTPOS_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn empty(side_to_move: Color) -> Self {
        Self {
            board: [None; BOARD_SQUARES],
            side_to_move,
            castling_rights: CastlingRights::default(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn starting_position() -> Self {
        let mut position = Self::empty(Color::White);
        position.castling_rights = CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        };
        let back_rank = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];
        for (file, kind) in back_rank.into_iter().enumerate() {
            position.set_piece(
                Square::from_file_rank(file as u8, 0).unwrap(),
                Some(Piece {
                    color: Color::White,
                    kind,
                }),
            );
            position.set_piece(
                Square::from_file_rank(file as u8, 1).unwrap(),
                Some(Piece {
                    color: Color::White,
                    kind: PieceKind::Pawn,
                }),
            );
            position.set_piece(
                Square::from_file_rank(file as u8, 6).unwrap(),
                Some(Piece {
                    color: Color::Black,
                    kind: PieceKind::Pawn,
                }),
            );
            position.set_piece(
                Square::from_file_rank(file as u8, 7).unwrap(),
                Some(Piece {
                    color: Color::Black,
                    kind,
                }),
            );
        }
        position
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut fields = fen.split_whitespace();
        let board_field = fields
            .next()
            .ok_or_else(|| "missing board field in FEN".to_string())?;
        let side_field = fields
            .next()
            .ok_or_else(|| "missing side-to-move field in FEN".to_string())?;
        let castling_field = fields.next().unwrap_or("-");
        let en_passant_field = fields.next().unwrap_or("-");
        let halfmove_clock = fields
            .next()
            .unwrap_or("0")
            .parse::<u16>()
            .map_err(|_| "invalid halfmove clock in FEN".to_string())?;
        let fullmove_number = fields
            .next()
            .unwrap_or("1")
            .parse::<u16>()
            .map_err(|_| "invalid fullmove number in FEN".to_string())?;

        let side_to_move = Color::from_fen_symbol(
            side_field
                .chars()
                .next()
                .ok_or_else(|| "empty side-to-move field in FEN".to_string())?,
        )
        .ok_or_else(|| "invalid side-to-move field in FEN".to_string())?;

        let castling_rights = CastlingRights::from_fen(castling_field)?;
        let en_passant_target = if en_passant_field == "-" {
            None
        } else {
            Some(
                Square::from_algebraic(en_passant_field)
                    .ok_or_else(|| "invalid en-passant square in FEN".to_string())?,
            )
        };

        let mut position = Self {
            board: [None; BOARD_SQUARES],
            side_to_move,
            castling_rights,
            en_passant_target,
            halfmove_clock,
            fullmove_number,
        };

        let ranks: Vec<&str> = board_field.split('/').collect();
        if ranks.len() != 8 {
            return Err("invalid board layout in FEN".to_string());
        }

        for (rank_index, rank_field) in ranks.iter().enumerate() {
            let board_rank = 7_u8.saturating_sub(rank_index as u8);
            let mut file = 0_u8;
            for symbol in rank_field.chars() {
                if symbol.is_ascii_digit() {
                    let empty_count = symbol
                        .to_digit(10)
                        .ok_or_else(|| "invalid digit in FEN".to_string())?
                        as u8;
                    file = file.saturating_add(empty_count);
                    continue;
                }
                if file >= 8 {
                    return Err("file overflow in FEN rank".to_string());
                }
                let piece = Piece::from_fen_symbol(symbol)
                    .ok_or_else(|| "invalid piece in FEN".to_string())?;
                let square = Square::from_file_rank(file, board_rank)
                    .ok_or_else(|| "invalid square while decoding FEN".to_string())?;
                position.set_piece(square, Some(piece));
                file += 1;
            }
            if file != 8 {
                return Err("rank does not fill eight files in FEN".to_string());
            }
        }

        Ok(position)
    }

    pub fn to_fen(&self) -> String {
        let mut ranks = Vec::with_capacity(8);
        for rank in (0..8).rev() {
            let mut rank_text = String::new();
            let mut empty_run = 0;
            for file in 0..8 {
                let square = Square::from_file_rank(file, rank).unwrap();
                if let Some(piece) = self.piece_at(square) {
                    if empty_run > 0 {
                        rank_text.push(char::from_digit(empty_run, 10).unwrap());
                        empty_run = 0;
                    }
                    rank_text.push(piece.fen_symbol());
                } else {
                    empty_run += 1;
                }
            }
            if empty_run > 0 {
                rank_text.push(char::from_digit(empty_run, 10).unwrap());
            }
            ranks.push(rank_text);
        }
        format!(
            "{} {} {} {} {} {}",
            ranks.join("/"),
            self.side_to_move.fen_symbol(),
            self.castling_rights.to_fen(),
            self.en_passant_target
                .map(|square| square.to_string())
                .unwrap_or_else(|| "-".to_string()),
            self.halfmove_clock,
            self.fullmove_number
        )
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.board[square.0 as usize]
    }

    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    pub fn en_passant_target(&self) -> Option<Square> {
        self.en_passant_target
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.board[square.0 as usize] = piece;
    }

    pub fn iter_pieces(&self) -> impl Iterator<Item = (Square, Piece)> + '_ {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(index, piece)| piece.map(|piece| (Square(index as u8), piece)))
    }

    pub fn king_square(&self, color: Color) -> Option<Square> {
        self.iter_pieces().find_map(|(square, piece)| {
            if piece.color == color && piece.kind == PieceKind::King {
                Some(square)
            } else {
                None
            }
        })
    }

    pub fn is_square_attacked(&self, target: Square, by_color: Color) -> bool {
        let pawn_rank_offset = match by_color {
            Color::White => -1,
            Color::Black => 1,
        };
        for pawn_file_offset in [-1, 1] {
            let file = target.file() as i8 + pawn_file_offset;
            let rank = target.rank() as i8 + pawn_rank_offset;
            if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                continue;
            }
            let square = Square::from_file_rank(file as u8, rank as u8).unwrap();
            if self.piece_at(square)
                == Some(Piece {
                    color: by_color,
                    kind: PieceKind::Pawn,
                })
            {
                return true;
            }
        }

        const KNIGHT_OFFSETS: [(i8, i8); 8] = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];
        for (df, dr) in KNIGHT_OFFSETS {
            let file = target.file() as i8 + df;
            let rank = target.rank() as i8 + dr;
            if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                continue;
            }
            let square = Square::from_file_rank(file as u8, rank as u8).unwrap();
            if self.piece_at(square)
                == Some(Piece {
                    color: by_color,
                    kind: PieceKind::Knight,
                })
            {
                return true;
            }
        }

        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let file = target.file() as i8 + df;
                let rank = target.rank() as i8 + dr;
                if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                    continue;
                }
                let square = Square::from_file_rank(file as u8, rank as u8).unwrap();
                if self.piece_at(square)
                    == Some(Piece {
                        color: by_color,
                        kind: PieceKind::King,
                    })
                {
                    return true;
                }
            }
        }

        let sliding_rays = [
            ((1, 0), PieceKind::Rook, PieceKind::Queen),
            ((-1, 0), PieceKind::Rook, PieceKind::Queen),
            ((0, 1), PieceKind::Rook, PieceKind::Queen),
            ((0, -1), PieceKind::Rook, PieceKind::Queen),
            ((1, 1), PieceKind::Bishop, PieceKind::Queen),
            ((1, -1), PieceKind::Bishop, PieceKind::Queen),
            ((-1, 1), PieceKind::Bishop, PieceKind::Queen),
            ((-1, -1), PieceKind::Bishop, PieceKind::Queen),
        ];
        for ((df, dr), primary, secondary) in sliding_rays {
            let mut file = target.file() as i8 + df;
            let mut rank = target.rank() as i8 + dr;
            while (0..8).contains(&file) && (0..8).contains(&rank) {
                let square = Square::from_file_rank(file as u8, rank as u8).unwrap();
                if let Some(piece) = self.piece_at(square) {
                    if piece.color == by_color && (piece.kind == primary || piece.kind == secondary)
                    {
                        return true;
                    }
                    break;
                }
                file += df;
                rank += dr;
            }
        }

        false
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let Some(king_square) = self.king_square(color) else {
            return false;
        };
        self.is_square_attacked(king_square, color.opposite())
    }

    pub fn apply_move(&self, chess_move: &ChessMove) -> Self {
        let mut next = self.clone();
        let moving_piece = next.piece_at(chess_move.from);
        let mut captured_piece = next.piece_at(chess_move.to);
        next.set_piece(chess_move.from, None);
        if let Some(mut piece) = moving_piece {
            if matches!(piece.kind, PieceKind::Pawn)
                && self.en_passant_target == Some(chess_move.to)
                && captured_piece.is_none()
                && chess_move.from.file() != chess_move.to.file()
            {
                let capture_rank = chess_move.from.rank();
                let capture_square =
                    Square::from_file_rank(chess_move.to.file(), capture_rank).unwrap();
                captured_piece = next.piece_at(capture_square);
                next.set_piece(capture_square, None);
            }
            if let Some(promotion) = chess_move.promotion {
                piece.kind = promotion;
            }
            if matches!(piece.kind, PieceKind::King) {
                match piece.color {
                    Color::White => {
                        next.castling_rights.white_kingside = false;
                        next.castling_rights.white_queenside = false;
                    }
                    Color::Black => {
                        next.castling_rights.black_kingside = false;
                        next.castling_rights.black_queenside = false;
                    }
                }
                let castle_distance = chess_move.to.file() as i8 - chess_move.from.file() as i8;
                if castle_distance.abs() == 2 {
                    let (rook_from_file, rook_to_file) =
                        if castle_distance > 0 { (7, 5) } else { (0, 3) };
                    let rank = chess_move.from.rank();
                    let rook_from = Square::from_file_rank(rook_from_file, rank).unwrap();
                    let rook_to = Square::from_file_rank(rook_to_file, rank).unwrap();
                    let rook = next.piece_at(rook_from);
                    next.set_piece(rook_from, None);
                    next.set_piece(rook_to, rook);
                }
            }
            if matches!(piece.kind, PieceKind::Rook) {
                match (piece.color, chess_move.from.file(), chess_move.from.rank()) {
                    (Color::White, 0, 0) => next.castling_rights.white_queenside = false,
                    (Color::White, 7, 0) => next.castling_rights.white_kingside = false,
                    (Color::Black, 0, 7) => next.castling_rights.black_queenside = false,
                    (Color::Black, 7, 7) => next.castling_rights.black_kingside = false,
                    _ => {}
                }
            }
            next.set_piece(chess_move.to, Some(piece));
            next.en_passant_target = if matches!(piece.kind, PieceKind::Pawn)
                && (chess_move.from.rank() as i8 - chess_move.to.rank() as i8).abs() == 2
            {
                Square::from_file_rank(
                    chess_move.from.file(),
                    ((chess_move.from.rank() as u16 + chess_move.to.rank() as u16) / 2) as u8,
                )
            } else {
                None
            };
            next.halfmove_clock =
                if matches!(piece.kind, PieceKind::Pawn) || captured_piece.is_some() {
                    0
                } else {
                    next.halfmove_clock.saturating_add(1)
                };
        }
        if let Some(captured_piece) = captured_piece {
            if matches!(captured_piece.kind, PieceKind::Rook) {
                match (
                    captured_piece.color,
                    chess_move.to.file(),
                    chess_move.to.rank(),
                ) {
                    (Color::White, 0, 0) => next.castling_rights.white_queenside = false,
                    (Color::White, 7, 0) => next.castling_rights.white_kingside = false,
                    (Color::Black, 0, 7) => next.castling_rights.black_queenside = false,
                    (Color::Black, 7, 7) => next.castling_rights.black_kingside = false,
                    _ => {}
                }
            }
        }
        next.side_to_move = self.side_to_move.opposite();
        if matches!(next.side_to_move, Color::White) {
            next.fullmove_number = next.fullmove_number.saturating_add(1);
        }
        next
    }

    pub fn apply_uci_move(&self, uci: &str) -> Option<Self> {
        let chess_move = ChessMove::from_uci(uci)?;
        self.legal_moves()
            .into_iter()
            .find(|candidate| candidate == &chess_move)
            .map(|candidate| self.apply_move(&candidate))
    }

    pub fn san_for_move(&self, chess_move: &ChessMove) -> Option<String> {
        let piece = self.piece_at(chess_move.from)?;
        let is_castling = piece.kind == PieceKind::King
            && chess_move.from.rank() == chess_move.to.rank()
            && (chess_move.from.file() as i8 - chess_move.to.file() as i8).abs() == 2;
        if is_castling {
            let mut san = if chess_move.to.file() > chess_move.from.file() {
                "O-O".to_string()
            } else {
                "O-O-O".to_string()
            };
            let next = self.apply_move(chess_move);
            match next.game_status() {
                GameStatus::Checkmate { .. } => san.push('#'),
                _ if next.is_in_check(next.side_to_move()) => san.push('+'),
                _ => {}
            }
            return Some(san);
        }

        let is_en_passant = self.en_passant_target == Some(chess_move.to)
            && self.piece_at(chess_move.to).is_none()
            && piece.kind == PieceKind::Pawn
            && chess_move.from.file() != chess_move.to.file();
        let is_capture = self.piece_at(chess_move.to).is_some() || is_en_passant;

        let mut san = String::new();
        if let Some(symbol) = piece.kind.san_symbol() {
            san.push(symbol);
            let ambiguity = self
                .legal_moves()
                .into_iter()
                .filter(|candidate| {
                    if candidate == chess_move {
                        return false;
                    }
                    let Some(candidate_piece) = self.piece_at(candidate.from) else {
                        return false;
                    };
                    candidate_piece.kind == piece.kind
                        && candidate_piece.color == piece.color
                        && candidate.to == chess_move.to
                        && candidate.promotion == chess_move.promotion
                })
                .collect::<Vec<_>>();
            if !ambiguity.is_empty() {
                let same_file = ambiguity
                    .iter()
                    .any(|candidate| candidate.from.file() == chess_move.from.file());
                let same_rank = ambiguity
                    .iter()
                    .any(|candidate| candidate.from.rank() == chess_move.from.rank());
                if !same_file {
                    san.push((b'a' + chess_move.from.file()) as char);
                } else if !same_rank {
                    san.push((b'1' + chess_move.from.rank()) as char);
                } else {
                    san.push((b'a' + chess_move.from.file()) as char);
                    san.push((b'1' + chess_move.from.rank()) as char);
                }
            }
        } else if is_capture {
            san.push((b'a' + chess_move.from.file()) as char);
        }

        if is_capture {
            san.push('x');
        }
        san.push_str(&chess_move.to.to_string());
        if let Some(promotion) = chess_move.promotion.and_then(|kind| kind.san_symbol()) {
            san.push('=');
            san.push(promotion);
        }

        let next = self.apply_move(chess_move);
        match next.game_status() {
            GameStatus::Checkmate { .. } => san.push('#'),
            _ if next.is_in_check(next.side_to_move()) => san.push('+'),
            _ => {}
        }
        Some(san)
    }

    pub fn pseudo_legal_moves(&self) -> Vec<ChessMove> {
        let mut moves = Vec::new();
        for (square, piece) in self.iter_pieces() {
            if piece.color != self.side_to_move {
                continue;
            }
            match piece.kind {
                PieceKind::Pawn => self.push_pawn_moves(square, piece.color, &mut moves),
                PieceKind::Knight => self.push_knight_moves(square, &mut moves),
                PieceKind::Bishop => self.push_slider_moves(
                    square,
                    &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    &mut moves,
                ),
                PieceKind::Rook => {
                    self.push_slider_moves(square, &[(1, 0), (-1, 0), (0, 1), (0, -1)], &mut moves)
                }
                PieceKind::Queen => self.push_slider_moves(
                    square,
                    &[
                        (1, 1),
                        (1, -1),
                        (-1, 1),
                        (-1, -1),
                        (1, 0),
                        (-1, 0),
                        (0, 1),
                        (0, -1),
                    ],
                    &mut moves,
                ),
                PieceKind::King => self.push_king_moves(square, &mut moves),
            }
        }
        moves
    }

    pub fn legal_moves(&self) -> Vec<ChessMove> {
        let side = self.side_to_move;
        self.pseudo_legal_moves()
            .into_iter()
            .filter(|chess_move| !self.apply_move(chess_move).is_in_check(side))
            .collect()
    }

    pub fn game_status(&self) -> GameStatus {
        if self.halfmove_clock >= 100 {
            return GameStatus::DrawByFiftyMoveRule;
        }
        let legal_moves = self.legal_moves();
        if !legal_moves.is_empty() {
            return GameStatus::Ongoing;
        }
        if self.is_in_check(self.side_to_move) {
            GameStatus::Checkmate {
                loser: self.side_to_move,
            }
        } else {
            GameStatus::Stalemate {
                side_to_move: self.side_to_move,
            }
        }
    }

    pub fn game_status_with_history(&self, repetition_history: &[u64]) -> GameStatus {
        let status = self.game_status();
        if status != GameStatus::Ongoing {
            return status;
        }
        let current_key = self.repetition_key();
        let current_count = repetition_history
            .iter()
            .filter(|candidate| **candidate == current_key)
            .count();
        if current_count >= 3 {
            GameStatus::DrawByRepetition
        } else {
            GameStatus::Ongoing
        }
    }

    pub fn static_eval(&self) -> f32 {
        let material_score = self
            .iter_pieces()
            .map(|(_, piece)| match piece.color {
                Color::White => piece.kind.centipawn_value(),
                Color::Black => -piece.kind.centipawn_value(),
            })
            .sum::<f32>();

        let piece_square_score = self
            .iter_pieces()
            .map(|(square, piece)| {
                let bonus = piece.kind.piece_square_bonus(square, piece.color);
                match piece.color {
                    Color::White => bonus,
                    Color::Black => -bonus,
                }
            })
            .sum::<f32>();

        let mobility_score = {
            let white_moves = self.clone_for_side(Color::White).pseudo_legal_moves().len() as f32;
            let black_moves = self.clone_for_side(Color::Black).pseudo_legal_moves().len() as f32;
            (white_moves - black_moves) * 2.0
        };

        let pawn_structure_score =
            self.pawn_structure_score(Color::White) - self.pawn_structure_score(Color::Black);

        let piece_coordination_score = self.piece_coordination_score(Color::White)
            - self.piece_coordination_score(Color::Black);

        let tactical_vulnerability_score = self
            .iter_pieces()
            .map(|(square, piece)| {
                if piece.kind == PieceKind::King {
                    return 0.0;
                }
                let attacked = self.is_square_attacked(square, piece.color.opposite());
                let defended = self.is_square_attacked(square, piece.color);
                if !attacked {
                    return 0.0;
                }
                let penalty = if defended {
                    piece.kind.centipawn_value() * 0.08
                } else {
                    piece.kind.centipawn_value() * 0.18
                };
                match piece.color {
                    Color::White => -penalty,
                    Color::Black => penalty,
                }
            })
            .sum::<f32>();

        let king_safety_score = match self.game_status() {
            GameStatus::Checkmate {
                loser: Color::White,
            } => -10_000.0,
            GameStatus::Checkmate {
                loser: Color::Black,
            } => 10_000.0,
            GameStatus::Stalemate { .. }
            | GameStatus::DrawByFiftyMoveRule
            | GameStatus::DrawByRepetition => 0.0,
            GameStatus::Ongoing => {
                let white_in_check = self.is_in_check(Color::White);
                let black_in_check = self.is_in_check(Color::Black);
                match (white_in_check, black_in_check) {
                    (true, false) => -35.0,
                    (false, true) => 35.0,
                    _ => 0.0,
                }
            }
        };

        let signed_score = material_score
            + piece_square_score
            + mobility_score
            + pawn_structure_score
            + piece_coordination_score
            + tactical_vulnerability_score
            + king_safety_score;
        match self.side_to_move {
            Color::White => signed_score,
            Color::Black => -signed_score,
        }
    }

    fn pawn_structure_score(&self, color: Color) -> f32 {
        let pawn_files = self.pawn_file_counts(color);
        let mut score = 0.0;
        for file in 0..8 {
            let pawns_on_file = pawn_files[file];
            if pawns_on_file > 1 {
                score -= (pawns_on_file as f32 - 1.0) * 18.0;
            }
            if pawns_on_file > 0 {
                let left_support = file > 0 && pawn_files[file - 1] > 0;
                let right_support = file < 7 && pawn_files[file + 1] > 0;
                if !left_support && !right_support {
                    score -= pawns_on_file as f32 * 14.0;
                }
            }
        }

        for (square, piece) in self.iter_pieces() {
            if piece.color != color || piece.kind != PieceKind::Pawn {
                continue;
            }
            if self.is_passed_pawn(square, color) {
                let advancement = match color {
                    Color::White => square.rank() as f32,
                    Color::Black => (7 - square.rank()) as f32,
                };
                score += 12.0 + advancement * 10.0;
            }
        }

        score
    }

    fn piece_coordination_score(&self, color: Color) -> f32 {
        let bishop_count = self
            .iter_pieces()
            .filter(|(_, piece)| piece.color == color && piece.kind == PieceKind::Bishop)
            .count();
        let bishop_pair_bonus = if bishop_count >= 2 { 28.0 } else { 0.0 };

        let own_pawn_files = self.pawn_file_counts(color);
        let enemy_pawn_files = self.pawn_file_counts(color.opposite());
        let rook_file_bonus = self
            .iter_pieces()
            .filter(|(_, piece)| piece.color == color && piece.kind == PieceKind::Rook)
            .map(|(square, _)| {
                let file = square.file() as usize;
                let own_pawns = own_pawn_files[file];
                let enemy_pawns = enemy_pawn_files[file];
                if own_pawns == 0 && enemy_pawns == 0 {
                    18.0
                } else if own_pawns == 0 {
                    10.0
                } else {
                    0.0
                }
            })
            .sum::<f32>();

        bishop_pair_bonus + rook_file_bonus
    }

    fn pawn_file_counts(&self, color: Color) -> [u8; 8] {
        let mut counts = [0u8; 8];
        for (square, piece) in self.iter_pieces() {
            if piece.color == color && piece.kind == PieceKind::Pawn {
                counts[square.file() as usize] = counts[square.file() as usize].saturating_add(1);
            }
        }
        counts
    }

    fn is_passed_pawn(&self, square: Square, color: Color) -> bool {
        for (candidate_square, piece) in self.iter_pieces() {
            if piece.color != color.opposite() || piece.kind != PieceKind::Pawn {
                continue;
            }
            let file_distance = (candidate_square.file() as i8 - square.file() as i8).abs();
            if file_distance > 1 {
                continue;
            }
            let blocks_path = match color {
                Color::White => candidate_square.rank() >= square.rank(),
                Color::Black => candidate_square.rank() <= square.rank(),
            };
            if blocks_path {
                return false;
            }
        }
        true
    }

    fn clone_for_side(&self, side_to_move: Color) -> Self {
        let mut cloned = self.clone();
        cloned.side_to_move = side_to_move;
        cloned
    }

    fn zobrist_like_key(&self) -> u64 {
        let mut key = self.repetition_key();
        key ^= (self.halfmove_clock as u64) << 32;
        key ^= self.fullmove_number as u64;
        key
    }

    pub fn repetition_key(&self) -> u64 {
        let mut key = 0xcbf2_9ce4_8422_2325u64;
        for (square, piece) in self.iter_pieces() {
            let piece_code = match (piece.color, piece.kind) {
                (Color::White, PieceKind::Pawn) => 0x11u64,
                (Color::White, PieceKind::Knight) => 0x12u64,
                (Color::White, PieceKind::Bishop) => 0x13u64,
                (Color::White, PieceKind::Rook) => 0x14u64,
                (Color::White, PieceKind::Queen) => 0x15u64,
                (Color::White, PieceKind::King) => 0x16u64,
                (Color::Black, PieceKind::Pawn) => 0x21u64,
                (Color::Black, PieceKind::Knight) => 0x22u64,
                (Color::Black, PieceKind::Bishop) => 0x23u64,
                (Color::Black, PieceKind::Rook) => 0x24u64,
                (Color::Black, PieceKind::Queen) => 0x25u64,
                (Color::Black, PieceKind::King) => 0x26u64,
            };
            let mix = ((square.0 as u64) << 8) ^ piece_code;
            key ^= mix.wrapping_mul(0x1000_0000_01b3);
            key = key.rotate_left(7).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }

        if self.side_to_move == Color::Black {
            key ^= 0xf0f0_f0f0_f0f0_f0f0;
        }
        if self.castling_rights.white_kingside {
            key ^= 0x0101_0101_0101_0101;
        }
        if self.castling_rights.white_queenside {
            key ^= 0x0202_0202_0202_0202;
        }
        if self.castling_rights.black_kingside {
            key ^= 0x0404_0404_0404_0404;
        }
        if self.castling_rights.black_queenside {
            key ^= 0x0808_0808_0808_0808;
        }
        if let Some(square) = self.en_passant_target {
            key ^= 0x1111_0000_0000_1111 ^ square.0 as u64;
        }
        key
    }

    fn push_move_if_valid(
        &self,
        from: Square,
        file: i8,
        rank: i8,
        moves: &mut Vec<ChessMove>,
    ) -> bool {
        if !(0..8).contains(&file) || !(0..8).contains(&rank) {
            return false;
        }
        let to = Square::from_file_rank(file as u8, rank as u8).unwrap();
        if let Some(piece) = self.piece_at(to) {
            if piece.color == self.side_to_move {
                return false;
            }
            moves.push(ChessMove::new(from, to));
            return false;
        }
        moves.push(ChessMove::new(from, to));
        true
    }

    fn push_slider_moves(&self, from: Square, directions: &[(i8, i8)], moves: &mut Vec<ChessMove>) {
        for (df, dr) in directions {
            let mut file = from.file() as i8 + df;
            let mut rank = from.rank() as i8 + dr;
            while self.push_move_if_valid(from, file, rank, moves) {
                file += df;
                rank += dr;
            }
        }
    }

    fn push_knight_moves(&self, from: Square, moves: &mut Vec<ChessMove>) {
        const OFFSETS: [(i8, i8); 8] = [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ];
        for (df, dr) in OFFSETS {
            self.push_move_if_valid(from, from.file() as i8 + df, from.rank() as i8 + dr, moves);
        }
    }

    fn push_king_moves(&self, from: Square, moves: &mut Vec<ChessMove>) {
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                self.push_move_if_valid(
                    from,
                    from.file() as i8 + df,
                    from.rank() as i8 + dr,
                    moves,
                );
            }
        }
        self.push_castling_moves(from, moves);
    }

    fn push_pawn_moves(&self, from: Square, color: Color, moves: &mut Vec<ChessMove>) {
        let direction = match color {
            Color::White => 1,
            Color::Black => -1,
        };
        let start_rank = match color {
            Color::White => 1,
            Color::Black => 6,
        };
        let next_rank = from.rank() as i8 + direction;
        if (0..8).contains(&next_rank) {
            let to = Square::from_file_rank(from.file(), next_rank as u8).unwrap();
            if self.piece_at(to).is_none() {
                let promotion_rank = matches!(color, Color::White) && next_rank == 7
                    || matches!(color, Color::Black) && next_rank == 0;
                if promotion_rank {
                    for promotion in PROMOTION_PIECES {
                        moves.push(ChessMove::new(from, to).with_promotion(promotion));
                    }
                } else {
                    moves.push(ChessMove::new(from, to));
                    if from.rank() == start_rank {
                        let skip_rank = next_rank + direction;
                        if (0..8).contains(&skip_rank) {
                            let skip_square =
                                Square::from_file_rank(from.file(), skip_rank as u8).unwrap();
                            if self.piece_at(skip_square).is_none() {
                                moves.push(ChessMove::new(from, skip_square));
                            }
                        }
                    }
                }
            }
        }
        for df in [-1, 1] {
            let file = from.file() as i8 + df;
            if !(0..8).contains(&file) || !(0..8).contains(&next_rank) {
                continue;
            }
            let to = Square::from_file_rank(file as u8, next_rank as u8).unwrap();
            let promotion_rank = matches!(color, Color::White) && next_rank == 7
                || matches!(color, Color::Black) && next_rank == 0;
            if let Some(piece) = self.piece_at(to) {
                if piece.color != color {
                    if promotion_rank {
                        for promotion in PROMOTION_PIECES {
                            moves.push(ChessMove::new(from, to).with_promotion(promotion));
                        }
                    } else {
                        moves.push(ChessMove::new(from, to));
                    }
                }
            } else if self.en_passant_target == Some(to) {
                moves.push(ChessMove::new(from, to));
            }
        }
    }

    fn push_castling_moves(&self, from: Square, moves: &mut Vec<ChessMove>) {
        let (color, rank, kingside_right, queenside_right) = match self.side_to_move {
            Color::White => (
                Color::White,
                0,
                self.castling_rights.white_kingside,
                self.castling_rights.white_queenside,
            ),
            Color::Black => (
                Color::Black,
                7,
                self.castling_rights.black_kingside,
                self.castling_rights.black_queenside,
            ),
        };
        if from != Square::from_file_rank(4, rank).unwrap() || self.is_in_check(color) {
            return;
        }
        if kingside_right
            && self
                .piece_at(Square::from_file_rank(5, rank).unwrap())
                .is_none()
            && self
                .piece_at(Square::from_file_rank(6, rank).unwrap())
                .is_none()
            && self.piece_at(Square::from_file_rank(7, rank).unwrap())
                == Some(Piece {
                    color,
                    kind: PieceKind::Rook,
                })
            && !self.is_square_attacked(Square::from_file_rank(5, rank).unwrap(), color.opposite())
            && !self.is_square_attacked(Square::from_file_rank(6, rank).unwrap(), color.opposite())
        {
            moves.push(ChessMove::new(
                from,
                Square::from_file_rank(6, rank).unwrap(),
            ));
        }
        if queenside_right
            && self
                .piece_at(Square::from_file_rank(1, rank).unwrap())
                .is_none()
            && self
                .piece_at(Square::from_file_rank(2, rank).unwrap())
                .is_none()
            && self
                .piece_at(Square::from_file_rank(3, rank).unwrap())
                .is_none()
            && self.piece_at(Square::from_file_rank(0, rank).unwrap())
                == Some(Piece {
                    color,
                    kind: PieceKind::Rook,
                })
            && !self.is_square_attacked(Square::from_file_rank(3, rank).unwrap(), color.opposite())
            && !self.is_square_attacked(Square::from_file_rank(2, rank).unwrap(), color.opposite())
        {
            moves.push(ChessMove::new(
                from,
                Square::from_file_rank(2, rank).unwrap(),
            ));
        }
    }
}

pub trait PolicyValueModel: Send + Sync {
    fn evaluate(&self, position: &Position) -> PolicyValue;
}

#[derive(Debug, Clone)]
pub struct TinyNeuralModel {
    pub input_weights: Vec<f32>,
    pub hidden_bias: Vec<f32>,
    pub value_weights: Vec<f32>,
    pub value_bias: f32,
    pub policy_weights: Vec<f32>,
    pub policy_bias: Vec<f32>,
}

impl Default for TinyNeuralModel {
    fn default() -> Self {
        let input_weights = (0..(TINY_INPUT_SIZE * TINY_HIDDEN_SIZE))
            .map(|index| ((index as f32 + 1.0) * 0.013).sin() * 0.08)
            .collect();
        let hidden_bias = (0..TINY_HIDDEN_SIZE)
            .map(|index| ((index as f32 + 1.0) * 0.17).cos() * 0.03)
            .collect();
        let value_weights = (0..TINY_HIDDEN_SIZE)
            .map(|index| ((index as f32 + 1.0) * 0.11).sin() * 0.09)
            .collect();
        let policy_weights = vec![0.0; TINY_HIDDEN_SIZE * TINY_MOVE_FEATURES];
        let policy_bias = vec![0.0; TINY_MOVE_FEATURES];
        Self {
            input_weights,
            hidden_bias,
            value_weights,
            value_bias: 0.0,
            policy_weights,
            policy_bias,
        }
    }
}

impl TinyNeuralModel {
    pub fn from_parameter_vector(vector: &[f32]) -> Result<Self, String> {
        let expected_len = Self::default().parameter_count();
        if vector.len() != expected_len {
            return Err(format!(
                "expected {expected_len} parameters, received {}",
                vector.len()
            ));
        }

        let input_weights_len = TINY_INPUT_SIZE * TINY_HIDDEN_SIZE;
        let hidden_bias_len = TINY_HIDDEN_SIZE;
        let value_weights_len = TINY_HIDDEN_SIZE;
        let policy_weights_len = TINY_HIDDEN_SIZE * TINY_MOVE_FEATURES;
        let policy_bias_len = TINY_MOVE_FEATURES;

        let mut offset = 0usize;
        let input_weights = vector[offset..offset + input_weights_len].to_vec();
        offset += input_weights_len;
        let hidden_bias = vector[offset..offset + hidden_bias_len].to_vec();
        offset += hidden_bias_len;
        let value_weights = vector[offset..offset + value_weights_len].to_vec();
        offset += value_weights_len;
        let value_bias = vector[offset];
        offset += 1;
        let policy_weights = vector[offset..offset + policy_weights_len].to_vec();
        offset += policy_weights_len;
        let policy_bias = vector[offset..offset + policy_bias_len].to_vec();

        Ok(Self {
            input_weights,
            hidden_bias,
            value_weights,
            value_bias,
            policy_weights,
            policy_bias,
        })
    }

    pub fn parameter_count(&self) -> usize {
        self.input_weights.len()
            + self.hidden_bias.len()
            + self.value_weights.len()
            + 1
            + self.policy_weights.len()
            + self.policy_bias.len()
    }

    pub fn parameter_vector(&self) -> Vec<f32> {
        let mut vector = Vec::with_capacity(self.parameter_count());
        vector.extend_from_slice(&self.input_weights);
        vector.extend_from_slice(&self.hidden_bias);
        vector.extend_from_slice(&self.value_weights);
        vector.push(self.value_bias);
        vector.extend_from_slice(&self.policy_weights);
        vector.extend_from_slice(&self.policy_bias);
        vector
    }

    pub fn encode_position(position: &Position) -> [f32; TINY_INPUT_SIZE] {
        let mut features = [0.0; TINY_INPUT_SIZE];
        for (square, piece) in position.iter_pieces() {
            let sign = match piece.color {
                Color::White => 1.0,
                Color::Black => -1.0,
            };
            let index = square.0 as usize;
            features[index] =
                sign * match piece.kind {
                    PieceKind::Pawn => 1.0,
                    PieceKind::Knight => 2.0,
                    PieceKind::Bishop => 3.0,
                    PieceKind::Rook => 4.0,
                    PieceKind::Queen => 5.0,
                    PieceKind::King => 6.0,
                } / 6.0;
        }
        features[95] = match position.side_to_move() {
            Color::White => 1.0,
            Color::Black => -1.0,
        };
        features
    }

    fn hidden_state(&self, position: &Position) -> Vec<f32> {
        let input = Self::encode_position(position);
        (0..TINY_HIDDEN_SIZE)
            .map(|hidden_index| {
                let base = hidden_index * TINY_INPUT_SIZE;
                let mut sum = self.hidden_bias[hidden_index];
                for input_index in 0..TINY_INPUT_SIZE {
                    sum += self.input_weights[base + input_index] * input[input_index];
                }
                sum / (1.0 + sum.abs())
            })
            .collect()
    }

    fn move_features(chess_move: &ChessMove, ply_hint: f32) -> [f32; TINY_MOVE_FEATURES] {
        let mut features = [0.0; TINY_MOVE_FEATURES];
        features[0] = chess_move.from.file() as f32 / 7.0;
        features[1] = chess_move.from.rank() as f32 / 7.0;
        features[2] = chess_move.to.file() as f32 / 7.0;
        features[3] = chess_move.to.rank() as f32 / 7.0;
        features[4] = if chess_move.promotion.is_some() {
            1.0
        } else {
            0.0
        };
        features[5] = ply_hint;
        features
    }
}

impl PolicyValueModel for TinyNeuralModel {
    fn evaluate(&self, position: &Position) -> PolicyValue {
        let hidden = self.hidden_state(position);
        let raw_value = hidden
            .iter()
            .zip(self.value_weights.iter())
            .fold(self.value_bias, |acc, (node, weight)| acc + (node * weight));
        let value = raw_value.tanh();

        let moves = position.legal_moves();
        if moves.is_empty() {
            let terminal_value = match position.game_status() {
                GameStatus::Checkmate { loser } if loser == position.side_to_move() => -1.0,
                GameStatus::Checkmate { .. } => 1.0,
                GameStatus::Stalemate { .. }
                | GameStatus::DrawByFiftyMoveRule
                | GameStatus::DrawByRepetition => 0.0,
                GameStatus::Ongoing => value,
            };
            return PolicyValue {
                value: terminal_value,
                policy: Vec::new(),
            };
        }

        let ply_hint = position.fullmove_number as f32 / 100.0;
        let mut entries: Vec<(ChessMove, f32)> = moves
            .into_iter()
            .map(|chess_move| {
                let move_features = Self::move_features(&chess_move, ply_hint);
                let mut score = 0.0;
                for hidden_index in 0..TINY_HIDDEN_SIZE {
                    let base = hidden_index * TINY_MOVE_FEATURES;
                    for feature_index in 0..TINY_MOVE_FEATURES {
                        score += hidden[hidden_index]
                            * self.policy_weights[base + feature_index]
                            * move_features[feature_index];
                    }
                }
                for feature_index in 0..TINY_MOVE_FEATURES {
                    score += self.policy_bias[feature_index] * move_features[feature_index];
                }
                (chess_move, score)
            })
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(DEFAULT_POLICY_WIDTH);
        let logits: Vec<f32> = entries
            .iter()
            .map(|(_, score)| score.clamp(-10.0, 10.0).exp())
            .collect();
        let total: f32 = logits.iter().sum::<f32>().max(f32::EPSILON);
        let policy = entries
            .into_iter()
            .zip(logits.into_iter())
            .map(|((chess_move, _), prior)| PolicyEntry {
                chess_move,
                prior: prior / total,
            })
            .collect();

        PolicyValue { value, policy }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchConfig {
    pub cpuct: f32,
    pub root_policy_width: usize,
    pub max_depth: usize,
    pub quiescence_depth: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            cpuct: 1.35,
            root_policy_width: DEFAULT_POLICY_WIDTH,
            max_depth: 2,
            quiescence_depth: 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MctsConfig {
    pub cpuct: f32,
    pub simulations: usize,
    pub root_policy_width: usize,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            cpuct: 1.35,
            simulations: 128,
            root_policy_width: DEFAULT_POLICY_WIDTH,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NesConfig {
    pub population_size: usize,
    pub sigma: f32,
    pub learning_rate: f32,
    pub seed: u64,
}

impl Default for NesConfig {
    fn default() -> Self {
        Self {
            population_size: 8,
            sigma: 0.05,
            learning_rate: 0.03,
            seed: 0x4E45_5301,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NesCaseResult {
    pub name: &'static str,
    pub chosen_move: Option<String>,
    pub preferred_hit: bool,
    pub avoided_hit: bool,
    pub preferred_policy_mass: f32,
    pub avoided_policy_mass: f32,
    pub value_alignment: f32,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct NesEvaluation {
    pub total_fitness: f32,
    pub cases: Vec<NesCaseResult>,
}

#[derive(Debug, Clone)]
pub struct NesStepResult {
    pub initial_fitness: f32,
    pub best_candidate_fitness: f32,
    pub updated_fitness: f32,
    pub evaluations: usize,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
struct NesCase {
    name: &'static str,
    fen: &'static str,
    preferred: &'static [&'static str],
    avoided: &'static [&'static str],
    target_value: f32,
}

const NES_CASES: &[NesCase] = &[
    NesCase {
        name: "mate-in-one",
        fen: "7k/R7/6K1/8/8/8/8/8 w - - 0 1",
        preferred: &["a7a8"],
        avoided: &[],
        target_value: 1.0,
    },
    NesCase {
        name: "win-free-queen",
        fen: "4k3/8/8/8/8/8/q7/R3K3 w - - 0 1",
        preferred: &["a1a2"],
        avoided: &[],
        target_value: 1.0,
    },
    NesCase {
        name: "save-hanging-queen",
        fen: "4k3/8/8/8/4r3/8/4Q3/4K3 w - - 0 1",
        preferred: &["e2f2", "e2d2", "e2c4"],
        avoided: &["e2e3", "e2d3"],
        target_value: 0.5,
    },
    NesCase {
        name: "white-opening-space",
        fen: Position::STARTPOS_FEN,
        preferred: &["e2e4", "d2d4", "c2c4", "g1f3"],
        avoided: &["a2a3", "h2h3", "a2a4", "h2h4"],
        target_value: 0.15,
    },
    NesCase {
        name: "black-central-reply",
        fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        preferred: &["c7c5", "e7e5", "e7e6", "c7c6"],
        avoided: &["a7a6", "h7h6", "a7a5", "h7h5"],
        target_value: 0.1,
    },
    NesCase {
        name: "promotion-race",
        fen: "4k3/6P1/8/8/8/8/8/4K3 w - - 0 1",
        preferred: &["g7g8q", "g7g8n"],
        avoided: &[],
        target_value: 1.0,
    },
];

#[derive(Debug, Clone)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn next_f32(&mut self) -> f32 {
        let value = self.next_u64() >> 40;
        (value as f32) / ((1u32 << 24) as f32)
    }

    fn standard_normal(&mut self) -> f32 {
        let u1 = self.next_f32().max(1.0e-7);
        let u2 = self.next_f32();
        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f32::consts::PI * u2;
        radius * theta.cos()
    }
}

pub fn evaluate_nes_fitness(
    vector: &[f32],
    mcts_config: MctsConfig,
) -> Result<NesEvaluation, String> {
    let model = TinyNeuralModel::from_parameter_vector(vector)?;
    let mut total_fitness = 0.0;
    let mut cases = Vec::with_capacity(NES_CASES.len());

    for nes_case in NES_CASES {
        let position = Position::from_fen(nes_case.fen)
            .map_err(|error| format!("invalid NES case FEN ({}): {error}", nes_case.name))?;
        let evaluation = model.evaluate(&position);
        let preferred_policy_mass = evaluation
            .policy
            .iter()
            .filter(|entry| {
                nes_case
                    .preferred
                    .iter()
                    .any(|uci| *uci == entry.chess_move.uci())
            })
            .map(|entry| entry.prior)
            .sum::<f32>();
        let avoided_policy_mass = evaluation
            .policy
            .iter()
            .filter(|entry| {
                nes_case
                    .avoided
                    .iter()
                    .any(|uci| *uci == entry.chess_move.uci())
            })
            .map(|entry| entry.prior)
            .sum::<f32>();
        let value_alignment = evaluation.value * nes_case.target_value;
        let engine = MctsEngine::new(model.clone(), mcts_config);
        let chosen_move = engine
            .best_move(&position)
            .map(|chess_move| chess_move.uci());
        let preferred_hit = chosen_move
            .as_ref()
            .map(|uci| nes_case.preferred.iter().any(|candidate| candidate == uci))
            .unwrap_or(false);
        let avoided_hit = chosen_move
            .as_ref()
            .map(|uci| nes_case.avoided.iter().any(|candidate| candidate == uci))
            .unwrap_or(false);

        let mut score = preferred_policy_mass * 1.25 - avoided_policy_mass * 0.75;
        score += value_alignment * 0.5;
        if preferred_hit {
            score += 2.0;
        }
        if avoided_hit {
            score -= 2.0;
        }

        total_fitness += score;
        cases.push(NesCaseResult {
            name: nes_case.name,
            chosen_move,
            preferred_hit,
            avoided_hit,
            preferred_policy_mass,
            avoided_policy_mass,
            value_alignment,
            score,
        });
    }

    Ok(NesEvaluation {
        total_fitness,
        cases,
    })
}

pub fn run_nes_step(
    base_vector: &[f32],
    nes_config: NesConfig,
    mcts_config: MctsConfig,
) -> Result<NesStepResult, String> {
    if base_vector.is_empty() {
        return Err("base NES vector was empty".to_string());
    }

    let initial = evaluate_nes_fitness(base_vector, mcts_config)?;
    let pair_count = (nes_config.population_size.max(2) + 1) / 2;
    let sigma = nes_config.sigma.max(1.0e-4);
    let mut rng = XorShift64::new(nes_config.seed);
    let mut gradient = vec![0.0; base_vector.len()];
    let mut best_candidate_fitness = initial.total_fitness;
    let mut best_vector = base_vector.to_vec();
    let mut evaluations = 0usize;

    for _ in 0..pair_count {
        let epsilon: Vec<f32> = (0..base_vector.len())
            .map(|_| rng.standard_normal())
            .collect();
        let plus_vector: Vec<f32> = base_vector
            .iter()
            .zip(epsilon.iter())
            .map(|(weight, noise)| weight + (noise * sigma))
            .collect();
        let minus_vector: Vec<f32> = base_vector
            .iter()
            .zip(epsilon.iter())
            .map(|(weight, noise)| weight - (noise * sigma))
            .collect();

        let plus_fitness = evaluate_nes_fitness(&plus_vector, mcts_config)?.total_fitness;
        let minus_fitness = evaluate_nes_fitness(&minus_vector, mcts_config)?.total_fitness;
        evaluations += 2;

        if plus_fitness > best_candidate_fitness {
            best_candidate_fitness = plus_fitness;
            best_vector = plus_vector.clone();
        }
        if minus_fitness > best_candidate_fitness {
            best_candidate_fitness = minus_fitness;
            best_vector = minus_vector.clone();
        }

        let delta = plus_fitness - minus_fitness;
        for (index, noise) in epsilon.iter().enumerate() {
            gradient[index] += delta * noise;
        }
    }

    let gradient_scale = nes_config.learning_rate / (pair_count as f32 * sigma);
    let mut updated_vector: Vec<f32> = base_vector
        .iter()
        .zip(gradient.iter())
        .map(|(weight, grad)| weight + (gradient_scale * grad))
        .collect();
    let updated_fitness = evaluate_nes_fitness(&updated_vector, mcts_config)?.total_fitness;
    evaluations += 1;
    if best_candidate_fitness > updated_fitness {
        updated_vector = best_vector;
    }
    let final_fitness = evaluate_nes_fitness(&updated_vector, mcts_config)?.total_fitness;
    evaluations += 1;

    Ok(NesStepResult {
        initial_fitness: initial.total_fitness,
        best_candidate_fitness,
        updated_fitness: final_fitness,
        evaluations,
        vector: updated_vector,
    })
}

#[derive(Debug, Clone)]
struct MctsChildStats {
    prior: f32,
    visits: u32,
    value_sum: f32,
}

impl MctsChildStats {
    fn mean_value(&self) -> f32 {
        if self.visits == 0 {
            0.0
        } else {
            self.value_sum / self.visits as f32
        }
    }
}

#[derive(Debug, Clone)]
struct MctsNode {
    visits: u32,
    children: HashMap<ChessMove, MctsChildStats>,
}

pub struct MctsEngine<M: PolicyValueModel> {
    model: M,
    config: MctsConfig,
    tree: RefCell<HashMap<u64, MctsNode>>,
}

impl<M: PolicyValueModel> MctsEngine<M> {
    pub fn new(model: M, config: MctsConfig) -> Self {
        Self {
            model,
            config,
            tree: RefCell::new(HashMap::new()),
        }
    }

    pub fn evaluate(&self, position: &Position) -> PolicyValue {
        let mut evaluation = self.model.evaluate(position);
        evaluation.policy.truncate(self.config.root_policy_width);
        evaluation
    }

    pub fn best_move(&self, position: &Position) -> Option<ChessMove> {
        let repetition_history = [position.repetition_key()];
        self.best_move_with_history(position, &repetition_history)
    }

    pub fn best_move_with_history(
        &self,
        position: &Position,
        repetition_history: &[u64],
    ) -> Option<ChessMove> {
        let legal_moves = position.legal_moves();
        if legal_moves.is_empty() {
            return None;
        }

        self.tree.borrow_mut().clear();
        for _ in 0..self.config.simulations.max(1) {
            let mut history = repetition_history.to_vec();
            let _ = self.simulate(position, &mut history);
        }

        let root_key = position.zobrist_like_key();
        let tree = self.tree.borrow();
        let root = tree.get(&root_key)?;
        root.children
            .iter()
            .max_by(|a, b| {
                a.1.visits
                    .cmp(&b.1.visits)
                    .then_with(|| {
                        a.1.mean_value()
                            .partial_cmp(&b.1.mean_value())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| a.0.uci().cmp(&b.0.uci()))
            })
            .map(|(chess_move, _)| chess_move.clone())
    }

    fn simulate(&self, position: &Position, repetition_history: &mut Vec<u64>) -> f32 {
        match position.game_status_with_history(repetition_history) {
            GameStatus::Checkmate { loser } => {
                return if loser == position.side_to_move() {
                    -1.0
                } else {
                    1.0
                };
            }
            GameStatus::Stalemate { .. }
            | GameStatus::DrawByFiftyMoveRule
            | GameStatus::DrawByRepetition => return 0.0,
            GameStatus::Ongoing => {}
        }

        let node_key = position.zobrist_like_key();
        let legal_moves = position.legal_moves();
        if legal_moves.is_empty() {
            return 0.0;
        }

        let needs_expansion = {
            let tree = self.tree.borrow();
            !tree.contains_key(&node_key)
        };
        if needs_expansion {
            let evaluation = self.evaluate(position);
            let mut children = HashMap::new();
            for chess_move in legal_moves {
                let prior = evaluation
                    .policy
                    .iter()
                    .find(|entry| entry.chess_move == chess_move)
                    .map(|entry| entry.prior)
                    .unwrap_or(1.0 / evaluation.policy.len().max(1) as f32);
                children.insert(
                    chess_move,
                    MctsChildStats {
                        prior,
                        visits: 0,
                        value_sum: 0.0,
                    },
                );
            }
            self.tree.borrow_mut().insert(
                node_key,
                MctsNode {
                    visits: 0,
                    children,
                },
            );
            return evaluation.value;
        }

        let selected_move = {
            let tree = self.tree.borrow();
            let node = tree.get(&node_key).expect("node should exist");
            let parent_visits = node.visits.max(1) as f32;
            node.children
                .iter()
                .max_by(|a, b| {
                    let a_score = self.puct_score(parent_visits, a.1);
                    let b_score = self.puct_score(parent_visits, b.1);
                    a_score
                        .partial_cmp(&b_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.0.uci().cmp(&b.0.uci()))
                })
                .map(|(chess_move, _)| chess_move.clone())
                .expect("expanded node must have children")
        };

        let child = position.apply_move(&selected_move);
        repetition_history.push(child.repetition_key());
        let child_value = -self.simulate(&child, repetition_history);
        repetition_history.pop();

        let mut tree = self.tree.borrow_mut();
        let node = tree.get_mut(&node_key).expect("node should exist");
        node.visits += 1;
        let child_stats = node
            .children
            .get_mut(&selected_move)
            .expect("selected child should exist");
        child_stats.visits += 1;
        child_stats.value_sum += child_value;
        child_value
    }

    fn puct_score(&self, parent_visits: f32, child: &MctsChildStats) -> f32 {
        let exploration =
            self.config.cpuct * child.prior * (parent_visits.sqrt() / (1.0 + child.visits as f32));
        child.mean_value() + exploration
    }
}

pub struct Engine<M: PolicyValueModel> {
    model: M,
    config: SearchConfig,
    table: RefCell<HashMap<(u64, usize), f32>>,
}

impl<M: PolicyValueModel> Engine<M> {
    pub fn new(model: M, config: SearchConfig) -> Self {
        Self {
            model,
            config,
            table: RefCell::new(HashMap::new()),
        }
    }

    pub fn evaluate(&self, position: &Position) -> PolicyValue {
        let mut evaluation = self.model.evaluate(position);
        evaluation.policy.truncate(self.config.root_policy_width);
        evaluation
    }

    pub fn best_move(&self, position: &Position) -> Option<ChessMove> {
        let repetition_history = [position.repetition_key()];
        self.best_move_with_history(position, &repetition_history)
    }

    pub fn best_move_with_history(
        &self,
        position: &Position,
        repetition_history: &[u64],
    ) -> Option<ChessMove> {
        let moves = self.ordered_moves(position);
        if moves.is_empty() {
            return None;
        }

        self.table.borrow_mut().clear();

        let mut principal_move = moves.first().cloned();
        for search_depth in 1..=self.config.max_depth.max(1) {
            let mut best_move = None;
            let mut best_score = f32::NEG_INFINITY;
            let mut alpha = f32::NEG_INFINITY;
            let beta = f32::INFINITY;
            let root_moves = self.ordered_root_moves(position, principal_move.clone());
            for chess_move in root_moves {
                let child = position.apply_move(&chess_move);
                let mut child_history = repetition_history.to_vec();
                child_history.push(child.repetition_key());
                let score = -self.negamax(
                    &child,
                    search_depth.saturating_sub(1),
                    -beta,
                    -alpha,
                    &child_history,
                );
                if score > best_score {
                    best_score = score;
                    best_move = Some(chess_move);
                }
                alpha = alpha.max(score);
            }
            if best_move.is_some() {
                principal_move = best_move;
            }
        }
        principal_move
    }

    fn ordered_moves(&self, position: &Position) -> Vec<ChessMove> {
        let legal_moves = position.legal_moves();
        if legal_moves.is_empty() {
            return legal_moves;
        }
        let priors = self.evaluate(position).policy;
        let mut scored_moves: Vec<(ChessMove, f32)> = legal_moves
            .into_iter()
            .map(|chess_move| {
                let prior = priors
                    .iter()
                    .find(|entry| entry.chess_move == chess_move)
                    .map(|entry| entry.prior)
                    .unwrap_or(0.0);
                let tactical_score = self.move_ordering_bonus(position, &chess_move);
                (chess_move, prior * 1000.0 + tactical_score)
            })
            .collect();
        scored_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_moves
            .into_iter()
            .map(|(chess_move, _)| chess_move)
            .collect()
    }

    fn ordered_root_moves(
        &self,
        position: &Position,
        principal_move: Option<ChessMove>,
    ) -> Vec<ChessMove> {
        let mut moves = self.ordered_moves(position);
        if let Some(principal_move) = principal_move {
            if let Some(index) = moves
                .iter()
                .position(|chess_move| *chess_move == principal_move)
            {
                moves.swap(0, index);
            }
        }
        moves
    }

    fn negamax(
        &self,
        position: &Position,
        depth: usize,
        mut alpha: f32,
        beta: f32,
        repetition_history: &[u64],
    ) -> f32 {
        match position.game_status_with_history(repetition_history) {
            GameStatus::Checkmate { .. } => return -10_000.0 + depth as f32,
            GameStatus::Stalemate { .. }
            | GameStatus::DrawByFiftyMoveRule
            | GameStatus::DrawByRepetition => return 0.0,
            GameStatus::Ongoing => {}
        }
        if depth == 0 {
            return self.quiescence(
                position,
                alpha,
                beta,
                self.config.quiescence_depth,
                repetition_history,
            );
        }

        let cache_key = (position.zobrist_like_key(), depth);
        if let Some(score) = self.table.borrow().get(&cache_key).copied() {
            return score;
        }

        let moves = self.ordered_moves(position);
        if moves.is_empty() {
            return self.leaf_score(position);
        }

        let mut best_score = f32::NEG_INFINITY;
        let mut completed_search = true;
        for chess_move in moves {
            let child = position.apply_move(&chess_move);
            let mut child_history = repetition_history.to_vec();
            child_history.push(child.repetition_key());
            let score = -self.negamax(&child, depth - 1, -beta, -alpha, &child_history);
            best_score = best_score.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                completed_search = false;
                break;
            }
        }
        if completed_search {
            self.table.borrow_mut().insert(cache_key, best_score);
        }
        best_score
    }

    fn leaf_score(&self, position: &Position) -> f32 {
        let model_score = self.evaluate(position).value * 100.0;
        let static_score = position.static_eval();
        static_score * 0.8 + model_score * 0.2
    }

    fn quiescence(
        &self,
        position: &Position,
        mut alpha: f32,
        beta: f32,
        depth: usize,
        repetition_history: &[u64],
    ) -> f32 {
        match position.game_status_with_history(repetition_history) {
            GameStatus::Checkmate { .. } => return -10_000.0 + depth as f32,
            GameStatus::Stalemate { .. }
            | GameStatus::DrawByFiftyMoveRule
            | GameStatus::DrawByRepetition => return 0.0,
            GameStatus::Ongoing => {}
        }

        let stand_pat = self.leaf_score(position);
        if stand_pat >= beta {
            return stand_pat;
        }
        alpha = alpha.max(stand_pat);

        if depth == 0 {
            return stand_pat;
        }

        let tactical_moves = self.ordered_tactical_moves(position);
        if tactical_moves.is_empty() {
            return stand_pat;
        }

        let mut best_score = stand_pat;
        for chess_move in tactical_moves {
            let child = position.apply_move(&chess_move);
            let mut child_history = repetition_history.to_vec();
            child_history.push(child.repetition_key());
            let score = -self.quiescence(&child, -beta, -alpha, depth - 1, &child_history);
            best_score = best_score.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
        best_score
    }

    fn move_ordering_bonus(&self, position: &Position, chess_move: &ChessMove) -> f32 {
        let Some(moving_piece) = position.piece_at(chess_move.from) else {
            return 0.0;
        };
        let moving_piece_value = moving_piece.kind.centipawn_value();
        let capture_square = if position.en_passant_target == Some(chess_move.to)
            && position.piece_at(chess_move.to).is_none()
            && chess_move.from.file() != chess_move.to.file()
        {
            Square::from_file_rank(chess_move.to.file(), chess_move.from.rank()).unwrap()
        } else {
            chess_move.to
        };
        let capture_bonus = position
            .piece_at(capture_square)
            .map(|piece| piece.kind.centipawn_value() - (moving_piece_value * 0.1))
            .unwrap_or(0.0);
        let promotion_bonus = chess_move
            .promotion
            .map(|promotion| promotion.centipawn_value())
            .unwrap_or(0.0);
        let after_position = position.apply_move(chess_move);
        let check_bonus = if after_position.is_in_check(position.side_to_move().opposite()) {
            75.0
        } else {
            0.0
        };
        let tactical_safety_bonus =
            self.tactical_move_safety_bonus(position, &after_position, chess_move, moving_piece);
        capture_bonus + promotion_bonus + check_bonus + tactical_safety_bonus
    }

    fn tactical_move_safety_bonus(
        &self,
        position: &Position,
        after_position: &Position,
        chess_move: &ChessMove,
        moving_piece: Piece,
    ) -> f32 {
        let own_color = moving_piece.color;
        let enemy_color = own_color.opposite();
        let moving_piece_value = moving_piece.kind.centipawn_value();

        let was_attacked = position.is_square_attacked(chess_move.from, enemy_color);
        let was_defended = position.is_square_attacked(chess_move.from, own_color);
        let escapes_hanging_piece = was_attacked && !was_defended;

        let is_attacked_after = after_position.is_square_attacked(chess_move.to, enemy_color);
        let is_defended_after = after_position.is_square_attacked(chess_move.to, own_color);

        let mut bonus = 0.0;
        if escapes_hanging_piece && (!is_attacked_after || is_defended_after) {
            bonus += moving_piece_value * 0.18;
        }
        if is_attacked_after && !is_defended_after {
            bonus -= moving_piece_value * 0.40;
        } else if is_attacked_after {
            bonus -= moving_piece_value * 0.08;
        }
        bonus
    }

    fn ordered_tactical_moves(&self, position: &Position) -> Vec<ChessMove> {
        self.ordered_moves(position)
            .into_iter()
            .filter(|chess_move| {
                chess_move.promotion.is_some()
                    || self.is_capture(position, chess_move)
                    || position
                        .apply_move(chess_move)
                        .is_in_check(position.side_to_move().opposite())
            })
            .collect()
    }

    fn is_capture(&self, position: &Position, chess_move: &ChessMove) -> bool {
        if position.piece_at(chess_move.to).is_some() {
            return true;
        }
        position.en_passant_target == Some(chess_move.to)
            && position.piece_at(chess_move.to).is_none()
            && chess_move.from.file() != chess_move.to.file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_has_pieces() {
        let position = Position::starting_position();
        assert_eq!(position.side_to_move(), Color::White);
        assert!(position
            .piece_at(Square::from_file_rank(4, 0).unwrap())
            .is_some());
        assert!(position
            .piece_at(Square::from_file_rank(4, 7).unwrap())
            .is_some());
    }

    #[test]
    fn pseudo_legal_moves_exist_from_start() {
        let position = Position::starting_position();
        let moves = position.pseudo_legal_moves();
        assert!(!moves.is_empty());
        assert!(moves.iter().any(|chess_move| chess_move.uci() == "b1c3"));
        assert!(moves.iter().any(|chess_move| chess_move.uci() == "e2e4"));
    }

    #[test]
    fn legal_moves_exist_from_start() {
        let position = Position::starting_position();
        let moves = position.legal_moves();
        assert!(!moves.is_empty());
        assert!(moves.iter().any(|chess_move| chess_move.uci() == "e2e4"));
    }

    #[test]
    fn neural_model_produces_policy_and_value() {
        let model = TinyNeuralModel::default();
        let evaluation = model.evaluate(&Position::starting_position());
        assert!(!evaluation.policy.is_empty());
        assert!((-1.0..=1.0).contains(&evaluation.value));
    }

    #[test]
    fn engine_returns_a_move() {
        let engine = Engine::new(TinyNeuralModel::default(), SearchConfig::default());
        let best_move = engine.best_move(&Position::starting_position());
        assert!(best_move.is_some());
    }

    #[test]
    fn mcts_engine_returns_a_move() {
        let engine = MctsEngine::new(
            TinyNeuralModel::default(),
            MctsConfig {
                simulations: 16,
                ..MctsConfig::default()
            },
        );
        let best_move = engine.best_move(&Position::starting_position());
        assert!(best_move.is_some());
    }

    #[test]
    fn engine_finds_mate_in_one() {
        let position = Position::from_fen("7k/R7/6K1/8/8/8/8/8 w - - 0 1").unwrap();
        let engine = Engine::new(
            TinyNeuralModel::default(),
            SearchConfig {
                max_depth: 2,
                ..SearchConfig::default()
            },
        );
        let best_move = engine.best_move(&position).unwrap();
        assert_eq!(best_move.uci(), "a7a8");
    }

    #[test]
    fn mcts_engine_finds_mate_in_one() {
        let position = Position::from_fen("7k/R7/6K1/8/8/8/8/8 w - - 0 1").unwrap();
        let engine = MctsEngine::new(
            TinyNeuralModel::default(),
            MctsConfig {
                simulations: 64,
                ..MctsConfig::default()
            },
        );
        let best_move = engine.best_move(&position).unwrap();
        assert_eq!(best_move.uci(), "a7a8");
    }

    #[test]
    fn static_eval_rewards_material_advantage() {
        let winning = Position::from_fen("4k3/8/8/8/8/8/Q7/4K3 w - - 0 1").unwrap();
        let equal = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(winning.static_eval() > equal.static_eval());
    }

    #[test]
    fn static_eval_rewards_piece_activity() {
        let centralized = Position::from_fen("4k3/8/8/3N4/8/8/8/4K3 w - - 0 1").unwrap();
        let rim = Position::from_fen("4k3/8/8/8/8/8/N7/4K3 w - - 0 1").unwrap();
        assert!(centralized.static_eval() > rim.static_eval());
    }

    #[test]
    fn static_eval_penalizes_hanging_pieces() {
        let hanging = Position::from_fen("4k3/8/8/8/4r3/8/4Q3/4K3 w - - 0 1").unwrap();
        let safe = Position::from_fen("4k3/8/8/8/8/8/4Q3/4K3 w - - 0 1").unwrap();
        assert!(safe.static_eval() > hanging.static_eval());
    }

    #[test]
    fn static_eval_rewards_passed_pawn() {
        let passed = Position::from_fen("4k3/8/8/3P4/8/8/8/4K3 w - - 0 1").unwrap();
        let blocked = Position::from_fen("4k3/3p4/8/3P4/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(passed.static_eval() > blocked.static_eval());
    }

    #[test]
    fn static_eval_rewards_bishop_pair() {
        let bishop_pair = Position::from_fen("4k3/8/8/8/8/8/3BB3/4K3 w - - 0 1").unwrap();
        let bishop_and_knight = Position::from_fen("4k3/8/8/8/8/8/3BN3/4K3 w - - 0 1").unwrap();
        assert!(bishop_pair.static_eval() > bishop_and_knight.static_eval());
    }

    #[test]
    fn move_ordering_rewards_escaping_hanging_queen() {
        let position = Position::from_fen("4k3/8/8/8/4r3/8/4Q3/4K3 w - - 0 1").unwrap();
        let engine = Engine::new(TinyNeuralModel::default(), SearchConfig::default());
        let safe_move = ChessMove::from_uci("e2f2").unwrap();
        let blunder_move = ChessMove::from_uci("e2e3").unwrap();
        assert!(
            engine.move_ordering_bonus(&position, &safe_move)
                > engine.move_ordering_bonus(&position, &blunder_move)
        );
    }

    #[test]
    fn san_formats_basic_pawn_and_piece_moves() {
        let position = Position::starting_position();
        assert_eq!(
            position
                .san_for_move(&ChessMove::from_uci("e2e4").unwrap())
                .as_deref(),
            Some("e4")
        );
        assert_eq!(
            position
                .san_for_move(&ChessMove::from_uci("g1f3").unwrap())
                .as_deref(),
            Some("Nf3")
        );
    }

    #[test]
    fn san_formats_castling_and_promotion() {
        let castling = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        assert_eq!(
            castling
                .san_for_move(&ChessMove::from_uci("e1g1").unwrap())
                .as_deref(),
            Some("O-O")
        );

        let promotion = Position::from_fen("4k3/6P1/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert_eq!(
            promotion
                .san_for_move(&ChessMove::from_uci("g7g8q").unwrap())
                .as_deref(),
            Some("g8=Q+")
        );
    }

    #[test]
    fn legal_moves_include_all_forward_promotion_variants() {
        let position = Position::from_fen("4k3/6P1/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let ucis = position
            .legal_moves()
            .into_iter()
            .map(|chess_move| chess_move.uci())
            .collect::<Vec<_>>();
        assert!(ucis.contains(&"g7g8q".to_string()));
        assert!(ucis.contains(&"g7g8r".to_string()));
        assert!(ucis.contains(&"g7g8b".to_string()));
        assert!(ucis.contains(&"g7g8n".to_string()));
    }

    #[test]
    fn legal_moves_include_all_capture_promotion_variants() {
        let position = Position::from_fen("4k2r/6P1/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let ucis = position
            .legal_moves()
            .into_iter()
            .map(|chess_move| chess_move.uci())
            .collect::<Vec<_>>();
        assert!(ucis.contains(&"g7h8q".to_string()));
        assert!(ucis.contains(&"g7h8r".to_string()));
        assert!(ucis.contains(&"g7h8b".to_string()));
        assert!(ucis.contains(&"g7h8n".to_string()));
    }

    #[test]
    fn engine_prefers_winning_queen_capture() {
        let position = Position::from_fen("4k3/8/8/8/8/8/q7/R3K3 w - - 0 1").unwrap();
        let engine = Engine::new(
            TinyNeuralModel::default(),
            SearchConfig {
                max_depth: 2,
                ..SearchConfig::default()
            },
        );
        let best_move = engine.best_move(&position).unwrap();
        assert_eq!(best_move.uci(), "a1a2");
    }

    #[test]
    fn search_scores_threefold_repetition_as_draw() {
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 4 3").unwrap();
        let engine = Engine::new(
            TinyNeuralModel::default(),
            SearchConfig {
                max_depth: 2,
                ..SearchConfig::default()
            },
        );
        let repetition_history = vec![
            position.repetition_key(),
            0xfeed_face_u64,
            position.repetition_key(),
            position.repetition_key(),
        ];
        assert_eq!(
            engine.negamax(
                &position,
                1,
                f32::NEG_INFINITY,
                f32::INFINITY,
                &repetition_history
            ),
            0.0
        );
    }

    #[test]
    fn parameter_vector_has_expected_shape() {
        let model = TinyNeuralModel::default();
        assert_eq!(
            model.parameter_count(),
            (TINY_INPUT_SIZE * TINY_HIDDEN_SIZE)
                + TINY_HIDDEN_SIZE
                + TINY_HIDDEN_SIZE
                + 1
                + (TINY_HIDDEN_SIZE * TINY_MOVE_FEATURES)
                + TINY_MOVE_FEATURES
        );
        assert_eq!(model.parameter_vector().len(), model.parameter_count());
    }

    #[test]
    fn parameter_vector_round_trips() {
        let model = TinyNeuralModel::default();
        let vector = model.parameter_vector();
        let rebuilt = TinyNeuralModel::from_parameter_vector(&vector).unwrap();
        assert_eq!(rebuilt.parameter_vector(), vector);
    }

    #[test]
    fn nes_fitness_evaluation_returns_cases() {
        let vector = TinyNeuralModel::default().parameter_vector();
        let evaluation = evaluate_nes_fitness(
            &vector,
            MctsConfig {
                simulations: 8,
                ..MctsConfig::default()
            },
        )
        .unwrap();
        assert_eq!(evaluation.cases.len(), NES_CASES.len());
        assert!(evaluation.total_fitness.is_finite());
    }

    #[test]
    fn nes_fitness_is_repeatable_for_same_vector() {
        let vector = TinyNeuralModel::default().parameter_vector();
        let config = MctsConfig {
            simulations: 8,
            ..MctsConfig::default()
        };
        let first = evaluate_nes_fitness(&vector, config).unwrap();
        let second = evaluate_nes_fitness(&vector, config).unwrap();
        assert_eq!(first.total_fitness, second.total_fitness);
        let first_moves = first
            .cases
            .iter()
            .map(|case| case.chosen_move.clone())
            .collect::<Vec<_>>();
        let second_moves = second
            .cases
            .iter()
            .map(|case| case.chosen_move.clone())
            .collect::<Vec<_>>();
        assert_eq!(first_moves, second_moves);
    }

    #[test]
    fn nes_step_preserves_vector_shape() {
        let vector = TinyNeuralModel::default().parameter_vector();
        let result = run_nes_step(
            &vector,
            NesConfig {
                population_size: 4,
                sigma: 0.03,
                learning_rate: 0.02,
                seed: 7,
            },
            MctsConfig {
                simulations: 8,
                ..MctsConfig::default()
            },
        )
        .unwrap();
        assert_eq!(result.vector.len(), vector.len());
        assert!(result.updated_fitness.is_finite());
        assert!(result.evaluations >= 3);
    }

    #[test]
    fn starting_position_round_trips_to_fen() {
        let position = Position::starting_position();
        assert_eq!(position.to_fen(), Position::STARTPOS_FEN);
        let reparsed = Position::from_fen(&position.to_fen()).unwrap();
        assert_eq!(reparsed.to_fen(), Position::STARTPOS_FEN);
    }

    #[test]
    fn parses_sparse_fen_position() {
        let position = Position::from_fen("8/8/3k4/8/8/4K3/8/8 b - - 4 18").unwrap();
        assert_eq!(position.side_to_move(), Color::Black);
        assert_eq!(
            position.piece_at(Square::from_file_rank(3, 5).unwrap()),
            Some(Piece {
                color: Color::Black,
                kind: PieceKind::King,
            })
        );
        assert_eq!(
            position.piece_at(Square::from_file_rank(4, 2).unwrap()),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::King,
            })
        );
        assert_eq!(position.to_fen(), "8/8/3k4/8/8/4K3/8/8 b - - 4 18");
    }

    #[test]
    fn preserves_castling_and_en_passant_in_fen() {
        let position =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1")
                .unwrap();
        assert_eq!(
            position.castling_rights(),
            CastlingRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: true,
                black_queenside: true,
            }
        );
        assert_eq!(position.en_passant_target(), Square::from_algebraic("d3"));
        assert_eq!(
            position.to_fen(),
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1"
        );
    }

    #[test]
    fn detects_check_and_filters_illegal_moves() {
        let position = Position::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").unwrap();
        assert!(position.is_in_check(Color::White));
        let moves = position.legal_moves();
        assert!(moves
            .iter()
            .all(|chess_move| chess_move.from == Square::from_algebraic("e1").unwrap()));
        assert!(moves.iter().any(|chess_move| chess_move.uci() == "e1e2"));
        assert!(!moves.iter().any(|chess_move| chess_move.uci() == "e1d2"));
    }

    #[test]
    fn parses_uci_moves() {
        let chess_move = ChessMove::from_uci("e2e4").unwrap();
        assert_eq!(chess_move.from, Square::from_file_rank(4, 1).unwrap());
        assert_eq!(chess_move.to, Square::from_file_rank(4, 3).unwrap());
        assert_eq!(chess_move.uci(), "e2e4");
    }

    #[test]
    fn applies_pseudo_legal_uci_move() {
        let position = Position::starting_position();
        let next = position.apply_uci_move("b1c3").unwrap();
        assert_eq!(next.side_to_move(), Color::Black);
        assert_eq!(
            next.piece_at(Square::from_file_rank(2, 2).unwrap()),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::Knight,
            })
        );
        assert!(next
            .piece_at(Square::from_file_rank(1, 0).unwrap())
            .is_none());
    }

    #[test]
    fn generates_and_applies_castling() {
        let position = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let moves = position.legal_moves();
        assert!(moves.iter().any(|chess_move| chess_move.uci() == "e1g1"));
        assert!(moves.iter().any(|chess_move| chess_move.uci() == "e1c1"));

        let kingside = position.apply_uci_move("e1g1").unwrap();
        assert_eq!(
            kingside.piece_at(Square::from_algebraic("g1").unwrap()),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::King,
            })
        );
        assert_eq!(
            kingside.piece_at(Square::from_algebraic("f1").unwrap()),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::Rook,
            })
        );
    }

    #[test]
    fn generates_and_applies_en_passant() {
        let position = Position::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
        let moves = position.legal_moves();
        assert!(moves.iter().any(|chess_move| chess_move.uci() == "e5d6"));

        let next = position.apply_uci_move("e5d6").unwrap();
        assert_eq!(
            next.piece_at(Square::from_algebraic("d6").unwrap()),
            Some(Piece {
                color: Color::White,
                kind: PieceKind::Pawn,
            })
        );
        assert!(next
            .piece_at(Square::from_algebraic("d5").unwrap())
            .is_none());
        assert_eq!(next.to_fen(), "4k3/8/3P4/8/8/8/8/4K3 b - - 0 1");
    }

    #[test]
    fn detects_checkmate_and_stalemate() {
        let checkmate = Position::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(
            checkmate.game_status(),
            GameStatus::Checkmate {
                loser: Color::Black,
            }
        );

        let stalemate = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(
            stalemate.game_status(),
            GameStatus::Stalemate {
                side_to_move: Color::Black,
            }
        );
    }

    #[test]
    fn detects_fifty_move_draw() {
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 51").unwrap();
        assert_eq!(position.game_status(), GameStatus::DrawByFiftyMoveRule);
    }

    #[test]
    fn detects_threefold_repetition_from_history() {
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 4 3").unwrap();
        let history = vec![
            position.repetition_key(),
            0x1234_u64,
            position.repetition_key(),
            position.repetition_key(),
        ];
        assert_eq!(
            position.game_status_with_history(&history),
            GameStatus::DrawByRepetition
        );
    }
}
