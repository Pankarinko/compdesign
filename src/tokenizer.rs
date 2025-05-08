use std::{str::from_utf8, u8};

enum Token<'a> {
    Identifier(&'a [u8]),
    NumericValue(i32),
    ArithmeticSymbol(ArithmeticSymbol),
    ArithmeticSymbolEqual(ArithmeticSymbolEqual),
    EndOfLine,
    BracketOpen,
    BracketClose,
    Keyword(Keyword),
}

enum ArithmeticSymbol {
    Plus,
    Minus,
    Mult,
    Div,
    Mod,
}

enum ArithmeticSymbolEqual {
    PlusEqual,
    MinusEqual,
    MultEqual,
    DivEqual,
    ModEqual,
    Equal,
}

enum Keyword {
    Struct,
    If,
    Else,
    While,
    For,
    Continue,
    Break,
    Return,
    Assert,
    True,
    False,
    Null,
    Print,
    Read,
    Alloc,
    AllocArray,
    Int,
    Bool,
    Void,
    Char,
    String,
}

/* Converts ASCII hex digits represented as u8 to the corresponding 32-bit integer */
fn convert_digit(digit: &u8) -> Option<i32> {
    Some(match digit {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'A' | b'a' => 10,
        b'B' | b'b' => 11,
        b'C' | b'c' => 12,
        b'D' | b'd' => 13,
        b'E' | b'e' => 14,
        b'F' | b'f' => 15,
        _ => return None,
    })
}

/* Converts a number string represented as u8 to i32 */
fn convert_u8_i32(word: &[u8]) -> Option<i32> {
    if word.len() > 2 && word[0] == 0 && (word[1] == b'x' || word[1] == b'X') {
        let mut i = 2;
        let mut hexval = 0;
        while i < word.len() {
            if let Some(digit) = convert_digit(&word[i]) {
                hexval = (hexval << 4) + digit;
                i += 1;
            } else {
                return None;
            }
        }
        return Some(hexval);
    } else {
        if word[0] == b'0' {
            if word.len() == 1 {
                return Some(0);
            } else {
                return None;
            }
        }
        let mut i = 1;
        let mut decval = 0;
        while i < word.len() {
            if let Some(digit) = convert_digit(&word[i]) {
                if digit <= 9 {
                    decval = decval * 10 + digit;
                    i += 1;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        return Some(decval);
    }
    None
}

fn tokenize<'a>(input_string: &'a [u8], tokens: &mut Vec<Token<'a>>) -> Result<i32, i32> {
    let end = input_string.len();
    let mut i = 0;
    loop {
        if i == end {
            return Ok(0);
        }
        let mut curr_end = i + 1;
        while curr_end < end && input_string[curr_end] != b' ' {
            curr_end += 1;
        }
        let word = &input_string[i..curr_end];
        i += word.len();
        if word.len() == 0 {
            break;
        }
        match word {
            b"+" => {
                tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Plus));
                break;
            }
            b"-" => {
                tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Minus));
                break;
            }
            b"*" => {
                tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Mult));
                break;
            }
            b"/" => {
                tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Div));
                break;
            }
            b"%" => {
                tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Mod));
            }
            b"+=" => {
                tokens.push(Token::ArithmeticSymbolEqual(
                    ArithmeticSymbolEqual::PlusEqual,
                ));
                break;
            }
            b"-=" => {
                tokens.push(Token::ArithmeticSymbolEqual(
                    ArithmeticSymbolEqual::MinusEqual,
                ));
                break;
            }
            b"*=" => {
                tokens.push(Token::ArithmeticSymbolEqual(
                    ArithmeticSymbolEqual::MultEqual,
                ));
                break;
            }
            b"/=" => {
                tokens.push(Token::ArithmeticSymbolEqual(
                    ArithmeticSymbolEqual::DivEqual,
                ));
                break;
            }
            b"%=" => {
                tokens.push(Token::ArithmeticSymbolEqual(
                    ArithmeticSymbolEqual::ModEqual,
                ));
            }
            b"=" => {
                tokens.push(Token::ArithmeticSymbolEqual(ArithmeticSymbolEqual::Equal));
                break;
            }
            b"struct" => {
                tokens.push(Token::Keyword(Keyword::Struct));
                break;
            }
            b"if" => {
                tokens.push(Token::Keyword(Keyword::If));
                break;
            }
            b"else" => {
                tokens.push(Token::Keyword(Keyword::Else));
                break;
            }
            b"while" => {
                tokens.push(Token::Keyword(Keyword::While));
                break;
            }
            b"for" => {
                tokens.push(Token::Keyword(Keyword::For));
                break;
            }
            b"continue" => {
                tokens.push(Token::Keyword(Keyword::Continue));
                break;
            }
            b"break" => {
                tokens.push(Token::Keyword(Keyword::Break));
                break;
            }
            b"return" => {
                tokens.push(Token::Keyword(Keyword::Return));
                break;
            }
            b"assert" => {
                tokens.push(Token::Keyword(Keyword::Assert));
                break;
            }
            b"true" => {
                tokens.push(Token::Keyword(Keyword::True));
                break;
            }
            b"false" => {
                tokens.push(Token::Keyword(Keyword::False));
                break;
            }
            b"NULL" => {
                tokens.push(Token::Keyword(Keyword::Null));
                break;
            }
            b"print" => {
                tokens.push(Token::Keyword(Keyword::Print));
                break;
            }
            b"read" => {
                tokens.push(Token::Keyword(Keyword::Read));
                break;
            }
            b"alloc" => {
                tokens.push(Token::Keyword(Keyword::Alloc));
                break;
            }
            b"alloc_array" => {
                tokens.push(Token::Keyword(Keyword::AllocArray));
                break;
            }
            b"int" => {
                tokens.push(Token::Keyword(Keyword::Int));
                break;
            }
            b"bool" => {
                tokens.push(Token::Keyword(Keyword::Bool));
                break;
            }
            b"void" => {
                tokens.push(Token::Keyword(Keyword::Void));
                break;
            }
            b"char" => {
                tokens.push(Token::Keyword(Keyword::Char));
                break;
            }
            b"string" => {
                tokens.push(Token::Keyword(Keyword::String));
                break;
            }
            _ => {}
        }
        /* Check if the word is a valid number */
        if let Some(number) = convert_u8_i32(word) {
            tokens.push(Token::NumericValue(number));
            break;
        }

        i += 1;
    }
    Ok(0)
}
