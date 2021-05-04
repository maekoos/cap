use std::fs;
use std::io;

#[derive(Debug, PartialEq, Clone)]
pub enum InputToken {
    Comment(String),
    StartParen,
    EndParen,
    Number(f64),
    Identifier(String),
}

pub fn lex_file(path: &str) -> io::Result<Vec<InputToken>> {
    let src = fs::read_to_string(path)?;
    let mut out = Vec::new();

    let append_src = |src: String, out: &mut Vec<InputToken>| {
        let src = src.trim().to_owned();
        if src == "" {
            return;
        }

        match src.parse::<f64>() {
            Ok(num) => out.push(InputToken::Number(num)),
            Err(_) => out.push(InputToken::Identifier(src)),
        }
    };

    let append_comment = |c: String, out: &mut Vec<InputToken>| {
        let c = c.trim().to_owned();
        if c == "" {
            return;
        }
        out.push(InputToken::Comment(c));
    };

    let mut current = String::new();
    let mut is_in_comment = false;
    for ch in src.chars() {
        if ch == ';' && !is_in_comment {
            is_in_comment = true;
            append_src(current, &mut out);
            current = String::new();
        } else if ch == '\n' && is_in_comment {
            is_in_comment = false;
            append_comment(current, &mut out);
            current = String::new();
        }

        if (ch.is_whitespace() || ch == '(' || ch == ')') && !is_in_comment {
            append_src(current, &mut out);
            current = String::new();
        }

        if ch == '(' && !is_in_comment {
            out.push(InputToken::StartParen);
        } else if ch == ')' && !is_in_comment {
            out.push(InputToken::EndParen);
        } else {
            current.push(ch);
        }
    }

    if is_in_comment {
        append_comment(current, &mut out);
    } else {
        append_src(current, &mut out);
    }

    Ok(out)
}
