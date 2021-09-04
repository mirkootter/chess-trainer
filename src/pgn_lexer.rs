#[derive(Clone, Copy, Debug)]
pub enum Token<'source> {
    StartVariation,
    EndVariation,
    SanMove(&'source [u8]),
    MoveNr
}

pub struct TokenIterator<'source>(&'source [u8]);

impl<'source> TokenIterator<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        TokenIterator(source)
    }
}

impl<'source> Iterator for TokenIterator<'source> {
    type Item = Token<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let input = self.0;
        let (input, _) = nom::bytes::complete::take_while::<_, _, nom::error::Error<_>>(nom::character::is_space)(input).unwrap();
        //gloo_console::log!("Huhu", std::str::from_utf8(&input[..10]).unwrap());
        if input.is_empty() {
            self.0 = input;
            None
        } else {
            let (input, token) = token(input).unwrap();
            self.0 = input;

            Some(token)
        }
    }
}

fn number(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    nom::bytes::complete::take_while1(nom::character::is_digit)(input)
}

fn san(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    nom::bytes::complete::take_while1(|ch| {
        match ch {
            b'a'..=b'z' => true,
            b'A'..=b'Z' => true,
            b'0'..=b'9' => true,
            b'-' => true,
            _ => false
        }
    })(input)
}

fn san_plus(input: &[u8]) -> nom::IResult<&[u8], Token> {
    let (input, result) = nom::combinator::recognize(|input| -> nom::IResult<&[u8], ()> {
        let (input, _) = san(input)?;
        let (input, _) = nom::bytes::complete::take_while(|ch| ch == b'+' || ch == b'#')(input)?;
        
        Ok((input, ()))
    })(input)?;

    Ok((input, Token::SanMove(result)))
}

fn move_number(input: &[u8]) -> nom::IResult<&[u8], Token> {
    let (input, _) = number(input)?;
    let (input, _) = nom::bytes::complete::take_while1(|ch| ch == b'.')(input)?;

    Ok((input, Token::MoveNr))
}

fn start_variation(input: &[u8]) -> nom::IResult<&[u8], Token> {
    let (input, _) = nom::bytes::complete::tag(b"(")(input)?;
    Ok((input, Token::StartVariation))
}

fn end_variation(input: &[u8]) -> nom::IResult<&[u8], Token> {
    let (input, _) = nom::bytes::complete::tag(b")")(input)?;
    Ok((input, Token::EndVariation))
}

fn token(input: &[u8]) -> nom::IResult<&[u8], Token> {
    nom::branch::alt((start_variation, end_variation, move_number, san_plus))(input)
}
