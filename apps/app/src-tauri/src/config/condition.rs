use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Var {
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug, Clone)]
pub enum Cond {
    Always,
    Cmp { var: Var, op: Op, rhs: f64 },
    And(Box<Cond>, Box<Cond>),
    Or(Box<Cond>, Box<Cond>),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    pub speed: f64,
}

impl Context {
    const fn get(&self, var: Var) -> f64 {
        match var {
            Var::Speed => self.speed,
        }
    }
}

impl Cond {
    #[must_use]
    pub fn eval(&self, ctx: &Context) -> bool {
        match self {
            Self::Always => true,
            Self::Cmp { var, op, rhs } => {
                let lhs = ctx.get(*var);
                match op {
                    Op::Lt => lhs < *rhs,
                    Op::Le => lhs <= *rhs,
                    Op::Gt => lhs > *rhs,
                    Op::Ge => lhs >= *rhs,
                    Op::Eq => (lhs - *rhs).abs() < f64::EPSILON,
                    Op::Ne => (lhs - *rhs).abs() >= f64::EPSILON,
                }
            }
            Self::And(l, r) => l.eval(ctx) && r.eval(ctx),
            Self::Or(l, r) => l.eval(ctx) || r.eval(ctx),
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub input: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid condition {:?}: {}", self.input, self.msg)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(input: &str) -> Result<Cond, ParseError> {
    let mut p = Parser {
        src: input,
        rest: input.trim(),
    };
    let cond = p.parse_or()?;
    if !p.rest.is_empty() {
        return Err(p.err(format!("unexpected trailing input {:?}", p.rest)));
    }
    Ok(cond)
}

struct Parser<'a> {
    src: &'a str,
    rest: &'a str,
}

impl<'a> Parser<'a> {
    fn err(&self, msg: impl Into<String>) -> ParseError {
        ParseError {
            msg: msg.into(),
            input: self.src.to_string(),
        }
    }

    fn skip_ws(&mut self) {
        self.rest = self.rest.trim_start();
    }

    fn eat(&mut self, s: &str) -> bool {
        self.skip_ws();
        if let Some(rem) = self.rest.strip_prefix(s) {
            self.rest = rem;
            true
        } else {
            false
        }
    }

    fn parse_or(&mut self) -> Result<Cond, ParseError> {
        let mut lhs = self.parse_and()?;
        while self.eat("||") {
            let rhs = self.parse_and()?;
            lhs = Cond::Or(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Cond, ParseError> {
        let mut lhs = self.parse_cmp()?;
        while self.eat("&&") {
            let rhs = self.parse_cmp()?;
            lhs = Cond::And(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_cmp(&mut self) -> Result<Cond, ParseError> {
        let var = self.parse_ident()?;
        let op = self.parse_op()?;
        let rhs = self.parse_number()?;
        Ok(Cond::Cmp { var, op, rhs })
    }

    fn parse_ident(&mut self) -> Result<Var, ParseError> {
        self.skip_ws();
        let end = self
            .rest
            .find(|c: char| !(c.is_ascii_alphabetic() || c == '_'))
            .unwrap_or(self.rest.len());
        if end == 0 {
            return Err(self.err("expected identifier"));
        }
        let (head, tail) = self.rest.split_at(end);
        let var = match head {
            "speed" => Var::Speed,
            other => return Err(self.err(format!("unknown identifier {other:?}"))),
        };
        self.rest = tail;
        Ok(var)
    }

    fn parse_op(&mut self) -> Result<Op, ParseError> {
        self.skip_ws();
        for (lit, op) in [
            ("<=", Op::Le),
            (">=", Op::Ge),
            ("==", Op::Eq),
            ("!=", Op::Ne),
            ("<", Op::Lt),
            (">", Op::Gt),
        ] {
            if let Some(rem) = self.rest.strip_prefix(lit) {
                self.rest = rem;
                return Ok(op);
            }
        }
        Err(self.err("expected comparison operator"))
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        self.skip_ws();
        let end = self
            .rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+'))
            .unwrap_or(self.rest.len());
        if end == 0 {
            return Err(self.err("expected number"));
        }
        let (head, tail) = self.rest.split_at(end);
        let n: f64 = head
            .parse()
            .map_err(|_| self.err(format!("bad number literal {head:?}")))?;
        self.rest = tail;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(speed: f64) -> Context {
        Context { speed }
    }

    fn eval(src: &str, speed: f64) -> bool {
        parse(src).expect("parse").eval(&ctx(speed))
    }

    #[test]
    fn basic_comparisons() {
        assert!(eval("speed < 200", 199.0));
        assert!(!eval("speed < 200", 200.0));
        assert!(eval("speed <= 200", 200.0));
        assert!(eval("speed >= 200", 200.0));
        assert!(!eval("speed > 200", 200.0));
        assert!(eval("speed != 200", 199.9));
        assert!(eval("speed == 200", 200.0));
    }

    #[test]
    fn conjunction() {
        assert!(eval("speed >= 200 && speed < 600", 400.0));
        assert!(!eval("speed >= 200 && speed < 600", 700.0));
        assert!(!eval("speed >= 200 && speed < 600", 100.0));
    }

    #[test]
    fn disjunction() {
        assert!(eval("speed < 100 || speed >= 500", 50.0));
        assert!(eval("speed < 100 || speed >= 500", 500.0));
        assert!(!eval("speed < 100 || speed >= 500", 300.0));
    }

    #[test]
    fn precedence_and_over_or() {
        assert!(eval("speed < 10 || speed >= 100 && speed < 200", 0.0));
        assert!(!eval("speed < 10 || speed >= 100 && speed < 200", 50.0));
        assert!(eval("speed < 10 || speed >= 100 && speed < 200", 150.0));
        assert!(!eval("speed < 10 || speed >= 100 && speed < 200", 250.0));
    }

    #[test]
    fn whitespace_insensitive() {
        assert!(eval("  speed<200  ", 100.0));
        assert!(eval("speed  >=  200  &&  speed  <  600", 400.0));
    }

    #[test]
    fn rejects_parentheses() {
        assert!(parse("(speed < 200)").is_err());
    }

    #[test]
    fn rejects_negation() {
        assert!(parse("!speed < 200").is_err());
    }

    #[test]
    fn rejects_unknown_identifier() {
        assert!(parse("bounced == 1").is_err());
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!(parse("speed < 200 garbage").is_err());
    }

    #[test]
    fn rejects_missing_rhs() {
        assert!(parse("speed <").is_err());
    }
}
