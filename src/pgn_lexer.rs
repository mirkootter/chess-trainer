// This file is part of the rust-pgn-tokenizer library.
//
// Copyright (C) 2017 Lakin Wecker <lakin@wecker.ca>
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//------------------------------------------------------------------------------
// Parsers for the SAN specification
//------------------------------------------------------------------------------

use nom::{
  IResult,
  error::{ErrorKind, ParseError},
  Err::{Error, Incomplete, Failure},
};
use std::fmt;

///-----------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Token <'a> {
  Move(&'a [u8]),
  NullMove(&'a [u8]),
  EscapeComment(&'a [u8]),
  NAG(&'a [u8]),
  MoveAnnotation(&'a [u8]),
  Result(&'a [u8]),
  Commentary(&'a [u8]),
  TagSymbol(&'a [u8]),
  TagString(&'a [u8]),
  StartVariation(&'a [u8]),
  EndVariation(&'a [u8]),
}

impl<'a> fmt::Display for Token<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      // The `f` value implements the `Write` trait, which is what the
      // write! macro is expecting. Note that this formatting ignores the
      // various flags provided to format strings.
      match *self {
          Token::Move(x) => write!(f, "Move({})", String::from_utf8_lossy(x)),
          Token::NullMove(x) => write!(f, "NullMove({})", String::from_utf8_lossy(x)),
          Token::EscapeComment(x) => write!(f, "EscapeComment({})", String::from_utf8_lossy(x)),
          Token::NAG(x) => write!(f, "NAG({})", String::from_utf8_lossy(x)),
          Token::MoveAnnotation(x) => write!(f, "MoveAnnotation({})", String::from_utf8_lossy(x)),
          Token::Result(x) => write!(f, "Result({})", String::from_utf8_lossy(x)),
          Token::Commentary(x) => write!(f, "Commentary({})", String::from_utf8_lossy(x)),
          Token::TagSymbol(x) => write!(f, "TagSymbol({})", String::from_utf8_lossy(x)),
          Token::TagString(x) => write!(f, "TagString({})", String::from_utf8_lossy(x)),
          Token::StartVariation(x) => write!(f, "StartVariation({})", String::from_utf8_lossy(x)),
          Token::EndVariation(x) => write!(f, "EndVariation({})", String::from_utf8_lossy(x)),
      }
  }
}

// TODO: Figure out better error handling
#[derive(Debug, PartialEq)]
pub enum PgnError<I> {
  SanPawnMoveInvalid,
  SanPieceMoveInvalid,
  UciPieceMoveInvalid,
  SanCastlesInvalid,
  SanNullMoveInvalid,
  SanEmptyInput,
  SanInvalidCharacter,
  PgnIntegerEmpty,
  PgnIntegerInvalid,
  PgnStringEmpty,
  PgnStringInvalid,
  PgnStringInvalidEscapeSequence,
  PgnStringTooLarge,
  PgnEscapeCommentEmpty,
  PgnEscapeCommentInvalid,
  PgnNagEmpty,
  PgnNagInvalid,
  PgnSymbolEmpty,
  PgnSymbolInvalid,
  PgnGameResultEmpty,
  PgnGameResultInvalid,
  PgnCommentaryEmpty,
  PgnCommentaryInvalid,
  PgnCommentaryTooLarge,
  PgnTagPairEmpty,
  PgnTagPairInvalid,
  PgnTagPairInvalidSymbol,
  PgnTagPairInvalidString,
  PgnMoveNumberEmpty,
  PgnMoveNumberInvalid,
  PgnStartVariationEmpty,
  PgnStartVariationInvalid,
  PgnEndVariationEmpty,
  PgnEndVariationInvalid,
  PgnMoveAnnotationEmpty,
  PgnMoveAnnotationInvalid,
  Nom(I, ErrorKind),
}


impl<I> ParseError<I> for PgnError<I> {
fn from_error_kind(input: I, kind: ErrorKind) -> Self {
  PgnError::Nom(input, kind)
}

fn append(_: I, _: ErrorKind, other: Self) -> Self {
  other
}
}



pub fn is_0(i:u8) -> bool { i == b'0' }
pub fn is_1(i:u8) -> bool { i == b'1' }
pub fn is_2(i:u8) -> bool { i == b'2' }
pub fn is_capture(i:u8) -> bool { i == b'x' }
pub fn is_dash(i:u8) -> bool { i == b'-' }
pub fn is_digit(i:u8) -> bool { i >= b'0' && i <= b'9' }
pub fn is_equals(i:u8) -> bool { i == b'=' }
pub fn is_file(i:u8) -> bool { i >= b'a' && i <= b'h' }
pub fn is_letter(i:u8) -> bool { is_lowercase_letter(i) || is_uppercase_letter(i) }
pub fn is_lowercase_letter(i:u8) -> bool { i >= b'a' && i <= b'z' }
pub fn is_o(i:u8) -> bool { i == b'O' }
pub fn is_period(i:u8) -> bool { i == b'.' }
pub fn is_piece(i:u8) -> bool { i == b'R' || i == b'N' || i == b'B' || i == b'Q' || i == b'K' }
pub fn is_plus_or_hash(i:u8) -> bool { i == b'#' || i == b'+' }
pub fn is_rank(i:u8) -> bool { i >= b'1' && i <= b'8' }
pub fn is_slash(i:u8) -> bool { i == b'/' }
pub fn is_space(i:u8) -> bool { i == b' ' }
pub fn is_star(i:u8) -> bool { i == b'*' }
pub fn is_uppercase_letter(i:u8) -> bool { i >= b'A' && i <= b'Z' }
pub fn is_whitespace(i:u8) -> bool { i == b' ' || i == b'\n' || i == b'\r' || i == b'\t'  }
pub fn is_zero(i:u8) -> bool { i == b'0' }

macro_rules! match_character {
  ($name:ident, $($matcher:ident),+) => {
      fn $name (incoming:&[u8]) -> Option<usize> {
          let mut i: usize = 0;
          $(
              {
                  if incoming.len() <= i || !$matcher(incoming[i]) {
                      return None
                  }
                  i += 1
              }
          )*;
          Some(i)
      }
  };
}

match_character![check, is_plus_or_hash];
match_character![pawn_capture, is_capture, is_file, is_rank];
match_character![pawn_move, is_rank];
match_character![promotion, is_equals, is_piece];

// e4 dxe4 e8=Q dxe8=Q
pub fn san_pawn_move(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  let rest = &i[1..];
  let result = pawn_capture(rest)
  .or_else(|| pawn_move(rest))
  .and_then(|length| {
      promotion(&rest[length..])
      .or_else(|| Some(0))
      .and_then(|l2| {
          let length = length + l2;
          check(&rest[length..])
          .or_else(|| Some(0))
          .and_then(|l3| Some(length + l3))
      })
  });
  match result {
      Some(length) => return Ok((&i[length+1..], Token::Move(&i[0..length+1]))),
      None => return Err(Error(PgnError::SanPawnMoveInvalid))
  }
}


// Ng1xf3
match_character![piece_capture_with_rank_and_file, is_file, is_rank, is_capture, is_file, is_rank];
// N1xf3
match_character![piece_capture_with_rank, is_rank, is_capture, is_file, is_rank];
// Ngxf3
match_character![piece_capture_with_file, is_file, is_capture, is_file, is_rank];
// Nxf3
match_character![piece_capture, is_capture, is_file, is_rank];
// Ng1f3
match_character![piece_move_with_rank_and_file, is_file, is_rank, is_file, is_rank];
// N1f3
match_character![piece_move_with_rank, is_rank, is_file, is_rank];
// Ngf3
match_character![piece_move_with_file, is_file, is_file, is_rank];
// Nf3
match_character![piece_move, is_file, is_rank];

pub fn san_piece_move(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  let rest = &i[1..];
  let result = piece_capture_with_rank_and_file(rest)
  .or_else(|| piece_capture_with_rank(rest))
  .or_else(|| piece_capture_with_file(rest))
  .or_else(|| piece_capture(rest))
  .or_else(|| piece_move_with_rank_and_file(rest))
  .or_else(|| piece_move_with_rank(rest))
  .or_else(|| piece_move_with_file(rest))
  .or_else(|| piece_move(rest))
  .and_then(|length| {
      check(&rest[length..])
      .or_else(|| Some(0))
      .and_then(|l2| Some(length + l2))
  });
  match result {
      Some(length) => return Ok((&i[length+1..], Token::Move(&i[0..length+1]))),
      None => return Err(Error(PgnError::SanPieceMoveInvalid))
  }
}

pub fn uci_piece_move(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  let result = piece_move_with_rank_and_file(i)
      .and_then(|length| {
          if length < i.len() {
              match i[length] {
                  b'p' | b'n' | b'b' | b'r' | b'q' => Some(length+1),
                  _ => Some(length)
              }
          } else {
              Some(length)
          }
      });

  match result {
      Some(length) => return Ok((&i[length..], Token::Move(&i[0..length]))),
      None => return Err(Error(PgnError::UciPieceMoveInvalid))
  }
}

match_character![king_side_castles, is_dash, is_o];
match_character![queen_side_castles, is_dash, is_o, is_dash, is_o];

pub fn san_castles(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  let rest = &i[1..];
  let result = queen_side_castles(rest)
  .or_else(|| king_side_castles(rest))
  .and_then(|length| {
      return check(&rest[length..])
      .or_else(|| Some(0))
      .and_then(|extra_length| Some(length+extra_length));
  });
  match result {
      Some(length) => return Ok((&i[length+1..], Token::Move(&i[0..length+1]))),
      None => return Err(Error(PgnError::SanCastlesInvalid))
  }
}

// Z0
match_character![null_move_z0, is_zero];
// --
match_character![null_move_dash_dash, is_dash];

pub fn san_null_move(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  let rest = &i[1..];
  let result = null_move_dash_dash(rest)
  .or_else(|| null_move_z0(rest))
  .and_then(|length| {
      return check(&rest[length..])
      .or_else(|| Some(0))
      .and_then(|extra_length| Some(length+extra_length));
  });
  match result {
      Some(length) => return Ok((&i[length+1..], Token::NullMove(&i[0..length+1]))),
      None => return Err(Error(PgnError::SanNullMoveInvalid))
  }
}


pub fn san_move_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::SanEmptyInput));
  }
  match i[0] {
      b'R' | b'N' | b'B' | b'Q' | b'K' => san_piece_move(i),
      b'a'..=b'h' => san_pawn_move(i),
      b'O' => san_castles(i),
      b'-' | b'Z' => san_null_move(i),
      _ => Err(Error(PgnError::SanInvalidCharacter))
  }
}

// Delimited by quote: ASCII 34
// \\ -> \
// \" -> "
// \t and \n not allowed
// max of 255 length
// printable characters, ASCII [32-126]
//
// Results still include \" and \\
//
// TODO: This does _not_ deal with utf-8
const MAX_LENGTH: usize = 255;
pub fn pgn_string(i:&[u8]) -> IResult<&[u8], &[u8], PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnStringEmpty));
  }
  let mut prev = i[0];
  if prev != b'"' {
      return Err(Error(PgnError::PgnStringInvalid));
  }
  let mut length = 1;
  while length < i.len() {
      let mut cur = i[length];
      if cur == b'"' && prev != b'\\' {
          break;
      } else if prev == b'\\' && (cur != b'\\' && cur != b'"') {
          return Err(Error(PgnError::PgnStringInvalidEscapeSequence));
      } else if prev == b'\\' && cur == b'\\' {
          cur = b' '; // fake that this isn't a \, because for escaping purposes it isn'
      }
      prev = cur;
      length += 1;
      if length > MAX_LENGTH {
          return Err(Error(PgnError::PgnStringTooLarge));
      }
      
  }
  // Ensure we skip over the quotes
  Ok((&i[length+1..], &i[1..length]))
}

pub fn pgn_integer(i:&[u8]) -> IResult<&[u8], &[u8], PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnIntegerEmpty));
  }
  let mut length = 0;
  while length < i.len() && i[length] >= b'0' && i[length] <= b'9' {
      length += 1
  }
  if length == 0 {
      return Err(Error(PgnError::PgnIntegerInvalid));
  } else {
      Ok((&i[length..], &i[0..length]))
  }
}

pub fn pgn_escape_comment_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnEscapeCommentEmpty));
  }
  if i[0] != b'%' {
      return Err(Error(PgnError::PgnEscapeCommentInvalid));
  }
  let mut length = 1;
  while length < i.len() && i[length] != b'\r' && i[length] != b'\n' {
      length += 1
  }
  Ok((&i[length..], Token::EscapeComment(&i[1..length])))
}

const MAX_MOVE_ANNOTATION_LENGTH:usize = 3;
pub fn pgn_move_annotation_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnMoveAnnotationEmpty));
  }
  if i[0] != b'?' && i[0] != b'!' {
      return Err(Error(PgnError::PgnMoveAnnotationInvalid));
  }
  let mut length = 1;
  while length < i.len() && length <= MAX_MOVE_ANNOTATION_LENGTH && (
      i[length] == b'?'
      || i[length] == b'!'
  ) {
      length += 1;
  }
  Ok((&i[length..], Token::MoveAnnotation(&i[0..length])))
}

pub fn pgn_nag_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnNagEmpty));
  }
  if i[0] != b'$' {
      return Err(Error(PgnError::PgnNagInvalid));
  }
  match pgn_integer(&i[1..]) {
      Ok((_left, integer)) => Ok((&i[integer.len()+1..], Token::NAG(&i[1..integer.len()+1]))),
      Err(Incomplete(x)) => Err(Incomplete(x)),
      _ => Err(Error(PgnError::PgnNagInvalid))
      
  }
}

pub fn pgn_symbol(i:&[u8]) -> IResult<&[u8], &[u8], PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnSymbolEmpty));
  }
  if !(is_digit(i[0]) || is_letter(i[0])) {
      return Err(Error(PgnError::PgnSymbolInvalid));
  }
  let mut length = 1;
  while length < i.len() && (
      is_digit(i[length])
      || is_letter(i[length])
      || is_plus_or_hash(i[length])
      || is_equals(i[length])
      || i[length] == b':'
      || i[length] == b'_'
      || i[length] == b'-'
  ) {
      length += 1
  }
  Ok((&i[length..], &i[0..length]))
}

match_character![game_ongoing, is_star];
match_character![game_white_win, is_1, is_dash, is_0];
match_character![game_black_win, is_0, is_dash, is_1];
match_character![game_draw, is_1, is_slash, is_2, is_dash, is_1, is_slash, is_2];

pub fn pgn_game_result_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnGameResultEmpty));
  }

  let result = game_ongoing(i)
  .or_else(|| game_white_win(i))
  .or_else(|| game_black_win(i))
  .or_else(|| game_draw(i));
  match result {
      Some(length) => return Ok((&i[length..], Token::Result(&i[0..length]))),
      None => return Err(Error(PgnError::PgnGameResultInvalid))
  }
}

// TODO: ; type comments

// This is somewhat arbitrarily picked to be 2MB. A single commentary token
// can't exceed that. We need to set _some_ sort of limit, this seems reasonable
const MAX_COMMENTARY_LENGTH:usize = 2097152;
pub fn pgn_commentary_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnCommentaryEmpty));
  }
  if i[0] != b'{' {
      return Err(Error(PgnError::PgnCommentaryInvalid));
  }
  let mut length = 1;
  while length < i.len() && i[length] != b'}' {
      length += 1;
      if length > MAX_COMMENTARY_LENGTH {
          return Err(Error(PgnError::PgnCommentaryTooLarge));
      }
  }
  // Ensure we skip over the braces.
  Ok((&i[length+1..], Token::Commentary(&i[1..length])))
}

pub fn remove_whitespace(i:&[u8]) -> &[u8] {
  let mut length = 0;
  while length < i.len() && is_whitespace(i[length]) {
      length += 1;
  }
  &i[length..]
}

pub fn get_spaces(i:&[u8]) -> &[u8] {
  let mut length = 0;
  while length < i.len() && is_space(i[length]) {
      length += 1;
  }
  &i[0..length]
}

pub fn pgn_tag_symbol_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>> {
  if i.len() < 1 {
      return Err(Error(PgnError::PgnTagPairEmpty));
  }
  if i[0] != b'[' {
      return Err(Error(PgnError::PgnTagPairInvalid));
  }
  let i = remove_whitespace(&i[1..]);
  match pgn_symbol(i) {
      Ok((i, symbol)) => {
          return Ok((&i[0..], Token::TagSymbol(symbol)));
      },
      Err(Incomplete(x)) => Err(Incomplete(x)),
      _ => Err(Error(PgnError::PgnTagPairInvalidSymbol))
  }
}

pub fn pgn_tag_string_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>> {
  if i.len() < 1 {
      return Err(Error(PgnError::PgnTagPairEmpty));
  }
  let i = remove_whitespace(&i[0..]);
  match pgn_string(i) {
      Ok((i, string)) => {
          let i = remove_whitespace(i);
          let x = 0;
          if x < i.len() && i[x] == b']' {
              return Ok((&i[1..], Token::TagString(string)));
          } else {
              return Err(Error(PgnError::PgnTagPairInvalid));
          }
      },
      Err(Incomplete(x)) => Err(Incomplete(x)),
      _ => Err(Error(PgnError::PgnTagPairInvalidString))
  }
}

match_character![one_period, is_period];
match_character![two_periods, is_period, is_period];
match_character![three_periods, is_period, is_period, is_period];

pub fn pgn_move_number(i:&[u8]) -> IResult<&[u8], &[u8], PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnMoveNumberEmpty));
  }
  let result = pgn_integer(i);
  match result {
      Ok((left, integer)) => {
          let ws = get_spaces(left);
          let left = &left[ws.len()..];
          let result = three_periods(left)
              .or_else(|| two_periods(left))
              .or_else(|| one_period(left))
              .or_else(|| Some(0)).
              and_then(|periods_length| Some(integer.len() + ws.len() + periods_length));
          match result {
              Some(length) => return Ok((&i[length..], &i[0..length])),
              None => return Ok((&i[integer.len()..], &i[0..integer.len()])),
          }
      },
      Err(Incomplete(x)) => Err(Incomplete(x)),
      _ => Err(Error(PgnError::PgnMoveNumberInvalid))
  }
}

pub fn pgn_start_variation_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnStartVariationEmpty));
  }
  if i[0] == b'(' {
      return Ok((&i[1..], Token::StartVariation(&i[0..1])));
  } else {
      return Err(Error(PgnError::PgnStartVariationInvalid));
  }
}

pub fn pgn_end_variation_token(i:&[u8]) -> IResult<&[u8], Token, PgnError<&str>>{
  if i.len() < 1 {
      return Err(Error(PgnError::PgnEndVariationEmpty));
  }
  if i[0] == b')' {
      return Ok((&i[1..], Token::EndVariation(&i[0..1])));
  } else {
      return Err(Error(PgnError::PgnEndVariationInvalid));
  }
}

pub fn or_else<I, O, E, Op>(res: IResult<I, O, E>, op: Op) -> IResult<I, O, E>
  where
      Op: FnOnce() -> IResult<I, O, E>, 
{
  match res {
      Ok((i, o)) => Ok((i, o)),
      Err(Incomplete(_)) => op(),
      Err(Error(_)) => op(),
      Err(Failure(_)) => op(),
  }
}

// A simple PGN token stream. Operates on a byte slice, and streams
// byte slices of the form Token::
pub struct PGNTokenIterator<'a> {
  bytes: &'a [u8],
}

impl<'a> PGNTokenIterator<'a> {
  pub fn new(bytes: &'a [u8]) -> PGNTokenIterator<'a> {
      PGNTokenIterator{bytes: bytes}
  }
}

// Implement `Iterator` for `Fibonacci`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl<'a> Iterator for PGNTokenIterator<'a> {
  type Item = Token<'a>;

  fn next(&mut self) -> Option<Token<'a>> {
      let i = self.bytes;
      let i = remove_whitespace(i);
      let mut result = pgn_escape_comment_token(i);
      result = or_else(result, || pgn_game_result_token(i));
      result = or_else(result, || {
          match pgn_move_number(i) {
              Ok((left, _)) => {
                  let left = remove_whitespace(left);
                  san_move_token(left)
              },
              Err(Incomplete(x)) => Err(Incomplete(x)),
              Err(Error(e)) => Err(Error(e)),
              Err(Failure(e)) => Err(Failure(e))
          }
      });
      result = or_else(result, || pgn_tag_symbol_token(i));
      result = or_else(result, || pgn_tag_string_token(i));
      result = or_else(result, || pgn_start_variation_token(i));
      result = or_else(result, || pgn_end_variation_token(i));
      result = or_else(result, || pgn_commentary_token(i));
      result = or_else(result, || pgn_nag_token(i));
      result = or_else(result, || pgn_move_annotation_token(i));
      result = or_else(result, || san_move_token(i));
      match result {
          Ok((i, token)) => {
              self.bytes = i;
              Some(token.clone())
          },
          _ => None
      }
  }
}