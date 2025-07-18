use std::fmt;

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Identifier(&'a [u8]),
    NumericValue(i32),
    ArithmeticSymbol(ArithmeticSymbol),
    ArithmeticSymbolEqual(ArithmeticSymbolEqual),
    StatementEnd,
    ParenthOpen,
    ParenthClose,
    BraceOpen,
    BraceClose,
    TernaryIf,
    TernaryThen,
    Comma,
    Keyword(Keyword),
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone)]
pub enum ArithmeticSymbol {
    Plus,
    Minus,
    Mult,
    Div,
    Mod,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    Not,
    BitNot,
}

#[derive(Debug, Clone)]
pub enum ArithmeticSymbolEqual {
    PlusEqual,
    MinusEqual,
    MultEqual,
    DivEqual,
    ModEqual,
    Equal,
    DoubleEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    NotEqual,
    BitOrEqual,
    BitAndEqual,
    BitXorEqual,
    LShiftEqual,
    RShiftEqual,
}

#[derive(Debug, Clone)]
pub enum Keyword {
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
    Flush,
}

/* Converts ASCII hex digits represented as u8 to the corresponding 32-bit integer */
fn convert_digit(digit: &u8) -> Option<u32> {
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

pub fn tokenize<'a>(
    input_string: &'a [u8],
    semantic_error: &mut bool,
    tokens: &mut Vec<Token<'a>>,
) -> Result<i32, i32> {
    let end = input_string.len();
    let mut i = 0;
    loop {
        if i == end {
            return Ok(0);
        }
        let equals = b'=';
        match input_string[i] {
            b',' => {
                tokens.push(Token::Comma);
                i += 1;
                continue;
            }
            b'?' => {
                tokens.push(Token::TernaryIf);
                i += 1;
                continue;
            }
            b':' => {
                tokens.push(Token::TernaryThen);
                i += 1;
                continue;
            }
            b'+' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::PlusEqual,
                    ));
                    i += 2;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Plus));
                    i += 1;
                }
                continue;
            }
            b'-' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::MinusEqual,
                    ));
                    i += 2;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Minus));
                    i += 1;
                }
                continue;
            }
            b'*' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::MultEqual,
                    ));
                    i += 2;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Mult));
                    i += 1;
                }
                continue;
            }
            b'/' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::DivEqual,
                    ));
                    i += 2;
                    continue;
                }
                if input_string[i + 1] == b'/' {
                    while i != end && input_string[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                } else if input_string[i + 1] == b'*' {
                    i += 2;
                    let mut open = 1;
                    while open > 0 {
                        if i + 1 >= end {
                            return Err(42);
                        }
                        if input_string[i] == b'/' && input_string[i + 1] == b'*' {
                            i += 1;
                            open += 1;
                        } else if input_string[i] == b'*' && input_string[i + 1] == b'/' {
                            i += 1;
                            open -= 1;
                        }
                        i += 1;
                    }
                    continue;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Div));
                    i += 1;
                }
                continue;
            }
            b'%' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::ModEqual,
                    ));
                    i += 2;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Mod));
                    i += 1;
                }
                continue;
            }
            b'=' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::DoubleEqual,
                    ));
                    i += 2;
                    continue;
                } else {
                    tokens.push(Token::ArithmeticSymbolEqual(ArithmeticSymbolEqual::Equal));
                    i += 1;
                    continue;
                }
            }
            b'(' => {
                tokens.push(Token::ParenthOpen);
                i += 1;
                continue;
            }
            b')' => {
                tokens.push(Token::ParenthClose);
                i += 1;
                continue;
            }
            b'{' => {
                tokens.push(Token::BraceOpen);
                i += 1;
                continue;
            }
            b'}' => {
                tokens.push(Token::BraceClose);
                i += 1;
                continue;
            }
            b';' => {
                tokens.push(Token::StatementEnd);
                i += 1;
                continue;
            }
            b'<' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::LessEqual,
                    ));
                    i += 2;
                    continue;
                } else if input_string[i + 1] == b'<' {
                    if input_string[i + 2] == equals {
                        tokens.push(Token::ArithmeticSymbolEqual(
                            ArithmeticSymbolEqual::LShiftEqual,
                        ));
                        i += 3;
                    } else {
                        tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::LShift));
                        i += 2;
                    }
                    continue;
                }

                tokens.push(Token::ArithmeticSymbolEqual(
                    ArithmeticSymbolEqual::LessThan,
                ));
                i += 1;
                continue;
            }
            b'>' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::GreaterEqual,
                    ));
                    i += 2;
                    continue;
                } else if input_string[i + 1] == b'>' {
                    if input_string[i + 2] == equals {
                        tokens.push(Token::ArithmeticSymbolEqual(
                            ArithmeticSymbolEqual::RShiftEqual,
                        ));
                        i += 3;
                    } else {
                        tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::RShift));
                        i += 2;
                    }
                    continue;
                }

                tokens.push(Token::ArithmeticSymbolEqual(
                    ArithmeticSymbolEqual::GreaterThan,
                ));
                i += 1;
                continue;
            }
            b'!' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::NotEqual,
                    ));
                    i += 2;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Not));
                    i += 1;
                    continue;
                }
                continue;
            }
            b'~' => {
                tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::BitNot));
                i += 1;
                continue;
            }
            b'&' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::BitAndEqual,
                    ));
                    i += 2;
                    continue;
                } else if input_string[i + 1] == b'&' {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::And));
                    i += 2;
                    continue;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::BitAnd));
                    i += 1;
                    continue;
                }
            }
            b'|' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::BitOrEqual,
                    ));
                    i += 2;
                } else if input_string[i + 1] == b'|' {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::Or));
                    i += 2;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::BitOr));
                    i += 1;
                    continue;
                }
                continue;
            }
            b'^' => {
                if input_string[i + 1] == equals {
                    tokens.push(Token::ArithmeticSymbolEqual(
                        ArithmeticSymbolEqual::BitXorEqual,
                    ));
                    i += 2;
                } else {
                    tokens.push(Token::ArithmeticSymbol(ArithmeticSymbol::BitXor));
                    i += 1;
                    continue;
                }
                continue;
            }

            b'\n' | b'\t' | b' ' => {
                i += 1;
                continue;
            }
            _ => {}
        }
        match input_string[i] {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let mut curr_end = i + 1;
                while curr_end < end {
                    match input_string[curr_end] {
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => {
                            curr_end += 1;
                        }
                        _ => {
                            break;
                        }
                    }
                }
                let word = &input_string[i..curr_end];
                i += word.len();
                match word {
                    b"struct" => {
                        tokens.push(Token::Keyword(Keyword::Struct));
                        continue;
                    }
                    b"if" => {
                        tokens.push(Token::Keyword(Keyword::If));
                        continue;
                    }
                    b"else" => {
                        tokens.push(Token::Keyword(Keyword::Else));
                        continue;
                    }
                    b"while" => {
                        tokens.push(Token::Keyword(Keyword::While));
                        continue;
                    }
                    b"for" => {
                        tokens.push(Token::Keyword(Keyword::For));
                        continue;
                    }
                    b"continue" => {
                        tokens.push(Token::Keyword(Keyword::Continue));
                        continue;
                    }
                    b"break" => {
                        tokens.push(Token::Keyword(Keyword::Break));
                        continue;
                    }
                    b"return" => {
                        tokens.push(Token::Keyword(Keyword::Return));
                        continue;
                    }
                    b"assert" => {
                        tokens.push(Token::Keyword(Keyword::Assert));
                        continue;
                    }
                    b"true" => {
                        tokens.push(Token::Keyword(Keyword::True));
                        continue;
                    }
                    b"false" => {
                        tokens.push(Token::Keyword(Keyword::False));
                        continue;
                    }
                    b"NULL" => {
                        tokens.push(Token::Keyword(Keyword::Null));
                        continue;
                    }
                    b"print" => {
                        tokens.push(Token::Keyword(Keyword::Print));
                        continue;
                    }
                    b"read" => {
                        tokens.push(Token::Keyword(Keyword::Read));
                        continue;
                    }
                    b"alloc" => {
                        tokens.push(Token::Keyword(Keyword::Alloc));
                        continue;
                    }
                    b"alloc_array" => {
                        tokens.push(Token::Keyword(Keyword::AllocArray));
                        continue;
                    }
                    b"int" => {
                        tokens.push(Token::Keyword(Keyword::Int));
                        continue;
                    }
                    b"bool" => {
                        tokens.push(Token::Keyword(Keyword::Bool));
                        continue;
                    }
                    b"void" => {
                        tokens.push(Token::Keyword(Keyword::Void));
                        continue;
                    }
                    b"char" => {
                        tokens.push(Token::Keyword(Keyword::Char));
                        continue;
                    }
                    b"string" => {
                        tokens.push(Token::Keyword(Keyword::String));
                        continue;
                    }
                    b"flush" => {
                        tokens.push(Token::Keyword(Keyword::Flush));
                        continue;
                    }
                    _ => {
                        tokens.push(Token::Identifier(word));
                        continue;
                    }
                }
            }
            b'0'..=b'9' => {
                if input_string[i] == b'0' {
                    i += 1;
                    if input_string[i] == b'x' || input_string[i] == b'X' {
                        i += 1;
                        let mut temp_i = 0;
                        let mut hexval: u32 = 0;
                        while let Some(digit) = convert_digit(&input_string[i + temp_i]) {
                            temp_i += 1;
                            if temp_i > 8 {
                                *semantic_error = true;
                            }

                            hexval = (hexval << 4) + digit;
                            if i + temp_i >= end {
                                break;
                            }
                        }
                        if temp_i == 0 {
                            return Err(42);
                        }
                        tokens.push(Token::NumericValue(hexval.cast_signed()));
                        i += temp_i;
                        continue;
                    } else {
                        tokens.push(Token::NumericValue(0));
                        continue;
                    }
                } else {
                    let mut decval: u32 = 0;
                    while let Some(digit) = convert_digit(&input_string[i]) {
                        if digit > 9 {
                            break;
                        }
                        i += 1;

                        if let Some(new_mul) = decval.checked_mul(10) {
                            if let Some(new_add) = new_mul.checked_add(digit) {
                                decval = new_add;
                                if decval > 0x80000000 {
                                    *semantic_error = true;
                                }
                                if i >= end {
                                    break;
                                }
                                continue;
                            }
                        }
                        *semantic_error = true;
                    }
                    tokens.push(Token::NumericValue(decval.cast_signed()));
                    continue;
                }
            }
            _ => return Err(42),
        }
    }
}
