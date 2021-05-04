use super::lex;

#[derive(Debug)]
pub enum ParseError {
    EOF(usize),
    UnexpectedToken(lex::InputToken, usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(f64),
    Identifier(String),
    SExpression(SExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct SExpression {
    body: Vec<Value>,
}

impl SExpression {
    pub fn new(body: Vec<Value>) -> Self {
        Self { body }
    }

    pub fn first(&self) -> Option<&Value> {
        self.body.first()
    }

    pub fn is_first(&self, comp: &Value) -> Option<bool> {
        let first = self.body.first()?;
        Some(first == comp)
    }

    pub fn body(&self) -> &Vec<Value> {
        &self.body
    }
}

pub fn parse(input: Vec<lex::InputToken>) -> Result<Vec<Value>, ParseError> {
    let input: Vec<lex::InputToken> = input
        .into_iter()
        .filter(|x| match x {
            lex::InputToken::Comment(_) => false,
            _ => true,
        })
        .collect();

    let mut cur_index = 0;
    let mut meat = Vec::new();
    loop {
        if input.len() == cur_index {
            break;
        }

        match parse_value(&input, cur_index) {
            Ok(v) => {
                meat.push(v.0);
                cur_index = v.1;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(meat)
}

fn parse_sexp(input: &[lex::InputToken], index: usize) -> Result<(SExpression, usize), ParseError> {
    match input.get(index + 0) {
        None => return Err(ParseError::EOF(2)),
        Some(lex::InputToken::StartParen) => {}
        Some(t) => return Err(ParseError::UnexpectedToken(t.clone(), index + 0)),
    }

    if input.get(index + 1) == Some(&lex::InputToken::EndParen) {
        return Ok((SExpression::new(Vec::new()), index + 2));
        // todo!("Return nil value?");
    }

    let mut body = Vec::new();
    let mut cur_index = index + 1;
    //TODO Parse value
    loop {
        let (value, new_index) = parse_value(&input, cur_index)?;
        body.push(value);

        cur_index = new_index;
        if input.get(cur_index + 0) == None {
            eprintln!("Missing end paren");
            return Err(ParseError::EOF(1));
        }
        if input.get(cur_index + 0) == Some(&lex::InputToken::EndParen) {
            cur_index += 1;
            break;
        }
    }

    let sexp = SExpression::new(body);
    return Ok((sexp, cur_index));
}

fn parse_value(input: &[lex::InputToken], index: usize) -> Result<(Value, usize), ParseError> {
    match &input.get(index + 0) {
        None => Err(ParseError::EOF(0)),
        Some(v) => match v {
            lex::InputToken::Number(v) => Ok((Value::Number(*v), index + 1)),
            lex::InputToken::Identifier(v) => Ok((Value::Identifier(v.to_owned()), index + 1)),
            lex::InputToken::StartParen => {
                let (expr, new_index) = parse_sexp(input, index)?;
                Ok((Value::SExpression(expr), new_index))
            }
            lex::InputToken::Comment(_) => unreachable!(),
            lex::InputToken::EndParen => Err(ParseError::UnexpectedToken(
                lex::InputToken::EndParen,
                index + 0,
            )),
        },
    }
}
