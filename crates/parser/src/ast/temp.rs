use regex::Regex;
use std::{collections::HashMap, fmt::Debug};


// # --- Lexer ---------------------------------------------------
//
// class Token:
//     def __init__(self, type_, value):
//         self.type = type_
//         self.value = value
//
//     def __repr__(self):
//         return f"Token({self.type}, {self.value!r})"
//
// def tokenize(text):
//     token_specification = [
//         ('NUMBER',   r'\d+(\.\d*)?'),   # Integer or decimal number.
//         ('ID',       r'[A-Za-z_]\w*'),   # Identifiers (or keywords).
//         ('OP',       r'[\+\-\*\/\^]'),   # Infix operators.
//         ('LPAREN',   r'\('),             # Left Parenthesis.
//         ('RPAREN',   r'\)'),             # Right Parenthesis.
//         ('SKIP',     r'[ \t]+'),         # Skip spaces and tabs.
//         ('MISMATCH', r'.'),              # Any other character.
//     ]
//     tok_regex = '|'.join(f'(?P<{name}>{pattern})' for name, pattern in token_specification)
//     for mo in re.finditer(tok_regex, text):
//         kind = mo.lastgroup
//         value = mo.group()
//         if kind == 'NUMBER':
//             yield Token('NUMBER', float(value))
//         elif kind == 'ID':
//             yield Token('ID', value)
//         elif kind == 'OP':
//             yield Token('OP', value)
//         elif kind == 'LPAREN':
//             yield Token('LPAREN', value)
//         elif kind == 'RPAREN':
//             yield Token('RPAREN', value)
//         elif kind == 'SKIP':
//             continue
//         else:
//             raise SyntaxError(f'Unexpected token: {value}')

pub enum Expr {
    Lit(Lit),
    Mixfix {
        /// Name of operator
        ident: Ident,
        /// List of arguments
        args: Vec<Expr>
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit(lit) => lit.fmt(f),
            Expr::Mixfix { ident, args } => write!(f, "( {ident} {} )", args.iter().map(Expr::to_string()).collect::<Vec<_>>().join(" ")),
        }
    }
}


const OPERATORS : HashMap<&str, Operator> = HashMap::from_iter([
    ("+", Operator::new("_+_")), // precedence": 10, "associativity": "left"},
     ("-", Operator::new("_-_")), // "precedence": 10, "associativity": "left"},
     ("*", Operator::new("_*_")), // "precedence": 20, "associativity": "left"},
     ("/", Operator::new("_/_")), // "precedence": 20, "associativity": "left"},
     ("^", Operator::new("_^_")), // "precedence": 30, "associativity": "right"},
            ("if_then_else_", Operator::new("if_then_else_")), // 5, right
]);


struct Parser {
    tokens: Vec<&'static str>,
    pos: usize,
}

impl Parser {
    pub fn current_token(&self) -> Option<&'static str> {
        self.tokens.get(self.pos)
    }

    pub fn next(&mut self, expected: Option<&str>) -> &'static str {
        match self.tokens.get(self.pos) {
            Some(token) if Some(token) == expected => {
                self.pos += 1;
                token
            }
            Some(token) => 
                    panic!("Expected token {expected}, got {token}"),
            None => 
            panic!("Unexpected end of input"),
        }
        
    }

    pub fn parse(self) -> Expr {
        let expr = self.parse_expression(0);
        if let Some(token) = self.current_token() {
            panic!("Expected EOF, found: {token}")
        }
        expr
    }

    /// Parse an expression using precedence climbing
    fn parse_expression(self, min_prec: usize) -> Expr {
        let left = self.parse_primary();

        loop {
            let token = self.current_token();

            match token {
                Some(token) if token.=> {}
                    _ => break,
            }

            if token && token.type == 'OP' and token.value in self.operators {
                op_info = self.operators[token.value]
                prec = op_info['precedence']
                assoc = op_info.get('associativity', 'left')
                if prec < min_prec:
                    break

                op_token = self.eat('OP')
                # For left-associative operators, the next expression must bind tighter.
                next_min_prec = prec + 1 if assoc == 'left' else prec
                right = self.parse_expression(next_min_prec)
                left = BinaryOp(left, op_token.value, right)
        }
            else {
                break
                }
        }

        left
    }

    def parse_primary(self):
        token = self.current_token()
        if token is None:
            raise SyntaxError("Unexpected end of input")

        # --- Mixfix operator check ---
        # If the current token is an ID, check if it might begin a mixfix operator.
        if token.type == 'ID':
            for op_name, op_def in self.mixfix_ops.items():
                pattern = op_def['pattern']
                # Check if the first literal in the mixfix pattern matches.
                if pattern[0] == token.value:
                    # Attempt to parse the mixfix operator.
                    return self.parse_mixfix(op_name, op_def)
            # If no mixfix operator is recognized, treat it as a simple identifier.
            self.eat('ID')
            return Identifier(token.value)

        # Number literal.
        if token.type == 'NUMBER':
            self.eat('NUMBER')
            return Number(token.value)

        # Parenthesized expression.
        if token.type == 'LPAREN':
            self.eat('LPAREN')
            expr = self.parse_expression(0)
            if self.current_token() is None or self.current_token().type != 'RPAREN':
                raise SyntaxError("Missing closing parenthesis")
            self.eat('RPAREN')
            return expr

        raise SyntaxError(f"Unexpected token: {token}")

    def parse_mixfix(self, op_name, op_def):
        """
        Parses a mixfix operator given its name and definition.
        Assumes that the current token matches the first literal of the pattern.
        """
        pattern = op_def['pattern']
        precedence = op_def['precedence']
        args = []
        # The pattern is a list like: ["if", "_", "then", "_", "else", "_"].
        # We iterate over the pattern. The first element is already matched.
        # For each subsequent element:
        #   - If it is a placeholder ("_"), parse an expression.
        #   - Otherwise, expect a literal (typically a keyword).
        # Note: We use the

        use super::{Ident, Lit}; mixfix operator's precedence when parsing each argument.
        # Consume the first literal.
        expected_literal = pattern[0]
        token = self.eat('ID', expected_literal)
        # Process the remaining parts of the pattern.
        for part in pattern[1:]:
            if part == "_":
                # Parse an argument expression.
                arg = self.parse_expression(precedence)
                args.append(arg)
            else:
                # Expect a literal token.
                self.eat('ID', part)
        return MixfixOp(op_name, args)
}


# --- Example Usage ---------------------------------------------------

if __name__ == '__main__':
    # Example input using both infix and mixfix operators.
    # This expression corresponds to:
    # if 3 + 4 then 10 * 2 else 42
    expr_input = "if 3 + 4 then 10 * 2 else 42"
    tokens = list(tokenize(expr_input))
    print("Tokens:", tokens)

    parser = Parser(tokens, OPERATORS, MIXFIX_OPERATORS)
    ast = parser.parse()
    print("AST:", ast)

enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    fn map_right<K, F: FnOnce(R) -> K>(self, f: F) -> Either<L, K> {
        match self {
            Self::Left(l) => Either::Left(l),
            Self::Right(r) => Either::Right(f(r)),
        }
    }
}

impl<A> Either<A, A> {
    fn into_inner(self) -> A {
        match self {
            Self::Left(a) | Self::Right(a) => a,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Associativity {
    Left,
    Right,
    None,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Fixity {
    Prefix,
    Infix(Associativity),
    Postfix,
    Closed,
}

#[derive(Debug, PartialEq)]
pub struct Operator {
    pub fixity: Fixity,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NamePart(Vec<Option<String>>);

#[derive(Debug, Clone)]
pub struct OperatorPart {
    pub fixity: Fixity,
    pub name: String,
    pub part: NamePart,
}

impl Operator {
    pub fn new(fixity: Fixity, name: &str) -> Operator {
        let re = Regex::new("_+").unwrap();
        let name = re.replace_all(name, "_").to_string();
        Operator { fixity, name }
    }
}

impl From<&str> for NamePart {
    fn from(s: &str) -> Self {
        let mut result = vec![];
        let mut last = 0;
        for (index, matched) in s.match_indices('_') {
            if last != index {
                result.push(Some(s[last..index].to_string()));
            }
            result.push(None);
            last = index + matched.len();
        }
        if last < s.len() {
            result.push(Some(s[last..].to_string()));
        }

        if let Some(None) = result.last() {
            result.pop();
        }

        if let Some(None) = result.first() {
            result.remove(0);
        }

        NamePart(result)
    }
}

impl OperatorPart {
    pub fn new(op: &Operator) -> Self {
        let name = op.name.to_string();
        let fixity = op.fixity;
        let part = op.name.as_str().into();
        Self { fixity, name, part }
    }
}

#[derive(Debug)]
pub struct PrecTable(Vec<Vec<OperatorPart>>);

impl PrecTable {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, ops: &[(&Operator, usize)]) {
        let mut ops = ops.to_vec();
        ops.sort_by_key(|&(_, x)| x);
        ops.reverse();

        let mut prev = usize::MAX;
        for (op, cur) in ops {
            if prev == cur {
                self.0.last_mut().unwrap().push(OperatorPart::new(op));
            } else {
                self.0.push(Vec::new());
                self.0.last_mut().unwrap().push(OperatorPart::new(op));
            }
            prev = cur;
        }

        if self.0.last().unwrap().is_empty() {
            self.0.pop();
        }
    }

    fn ops(&self, prec: usize, fix: Fixity) -> Vec<&OperatorPart> {
        self.0[prec].iter().filter(|o| o.fixity == fix).collect()
        // self.0
        //     .iter()
        //     .flatten()
        //     .filter(|o| o.fixity == fix)
        //     .collect()
    }

    fn succ(&self, prec: usize) -> Vec<usize> {
        if prec + 1 < self.0.len() {
            vec![prec + 1]
        } else {
            Vec::new()
        }
        // (prec + 1..self.0.len()).collect()
    }

    fn all(&self) -> Vec<usize> {
        (0..self.0.len()).collect()
    }
}

pub type ParserResult<I, O, E> = Result<(I, O), E>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(String),
    UnexpectedEndOfInput,
    UnparsedInput(String),
}

pub trait Parser<'i, 's, O> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], O, ParseError>;
}

impl<'i, 's, O, P> Parser<'i, 's, O> for &P
where
    P: Parser<'i, 's, O>,
{
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], O, ParseError> {
        (*self).parse(i)
    }
}

impl<'i, 's, O> Parser<'i, 's, O> for &dyn Parser<'i, 's, O> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], O, ParseError> {
        (*self).parse(i)
    }
}

pub struct Tag(String);

impl<'i, 's> Parser<'i, 's, String> for Tag {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], String, ParseError> {
        if let Some(m) = i.first() {
            if *m == self.0 {
                Ok((&i[1..], self.0.clone()))
            } else {
                Err(ParseError::UnexpectedToken(m.to_string()))
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }
}

impl<'i, 's, O1, O2, P1, P2> Parser<'i, 's, (O1, O2)> for (P1, P2)
where
    P1: Parser<'i, 's, O1>,
    P2: Parser<'i, 's, O2>,
{
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], (O1, O2), ParseError> {
        let (p1, p2) = self;
        let (rest, o1) = p1.parse(i)?;
        let (rest, o2) = p2.parse(rest)?;
        Ok((rest, (o1, o2)))
    }
}
impl<'i, 's, O1, O2, O3, P1, P2, P3> Parser<'i, 's, (O1, O2, O3)> for (P1, P2, P3)
where
    P1: Parser<'i, 's, O1>,
    P2: Parser<'i, 's, O2>,
    P3: Parser<'i, 's, O3>,
{
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], (O1, O2, O3), ParseError> {
        let (p1, p2, p3) = self;
        let (rest, o1) = p1.parse(i)?;
        let (rest, o2) = p2.parse(rest)?;
        let (rest, o3) = p3.parse(rest)?;
        Ok((rest, (o1, o2, o3)))
    }
}

struct Or<P1, P2>(P1, P2);
impl<'i, 's, O1, O2, P1, P2> Parser<'i, 's, Either<O1, O2>> for Or<P1, P2>
where
    P1: Parser<'i, 's, O1>,
    P2: Parser<'i, 's, O2>,
{
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Either<O1, O2>, ParseError> {
        let (p1, p2) = (&self.0, &self.1);
        match (p1.parse(i), p2.parse(i)) {
            (Ok((rest1, o1)), Ok((rest2, o2))) => {
                if rest1.len() < rest2.len() {
                    Ok((rest1, Either::Left(o1)))
                } else {
                    Ok((rest2, Either::Right(o2)))
                }
            }
            (Ok((rest, o1)), _) => Ok((rest, Either::Left(o1))),
            (_, Ok((rest, o2))) => Ok((rest, Either::Right(o2))),
            _ => Err(ParseError::UnexpectedToken(
                i.first().unwrap_or(&"").to_string(),
            )),
        }
    }
}

struct Many1<P>(P);

impl<'i, 's, O, P> Parser<'i, 's, Vec<O>> for Many1<P>
where
    P: Parser<'i, 's, O>,
{
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Vec<O>, ParseError> {
        let (mut s, o) = self.0.parse(i)?;
        let mut v = vec![o];
        while let Ok((rest, o)) = self.0.parse(s) {
            s = rest;
            v.push(o);
        }
        Ok((s, v))
    }
}

#[derive(Debug)]
pub struct Tree {
    pub op: OperatorPart,
    pub input: Vec<Tree>,
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "( {} ", self.op.name)?;
        for input in &self.input {
            write!(f, "{input} ")?;
        }
        write!(f, ")")
    }
}

struct Expr<'a>(&'a PrecTable);

impl<'i, 'a, 's> Parser<'i, 's, Tree> for Expr<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        self.0
            .all()
            .iter()
            .map(|&p| (self.0, p))
            .filter_map(|p| {
                [
                    Closed(p.0, p.1).parse(i),
                    InfixNonAssoc(p.0, p.1).parse(i),
                    PreRight(p.0, p.1).parse(i),
                    PostLeft(p.0, p.1).parse(i),
                ]
                .into_iter()
                .filter_map(|p| p.ok())
                .min_by_key(|(rest, _)| rest.len())
                .ok_or_else(|| ParseError::UnexpectedToken(i.first().unwrap_or(&"").to_string()))
                .ok()
            })
            .min_by_key(|(rest, _)| rest.len())
            .ok_or_else(|| ParseError::UnexpectedToken(i.first().unwrap_or(&"").to_string()))
    }
}

struct Op<'a>(&'a PrecTable, &'a OperatorPart);

impl<'a, 'i, 's> Parser<'i, 's, Tree> for Op<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        let op = self.1;
        let parts = &op.part.0;
        let mut vec = vec![];
        let mut next = i;
        for part in parts {
            if let Some(s) = part {
                let (rest, _) = Tag(s.clone()).parse(next)?;
                next = rest;
            } else {
                // hole
                let (rest, o) = Expr(self.0).parse(next)?;
                vec.push(o);
                next = rest;
            }
        }

        Ok((
            next,
            Tree {
                op: op.clone(),
                input: vec,
            },
        ))
    }
}

struct Inner<'a>(&'a PrecTable, usize, Fixity);

impl<'i, 'a, 's> Parser<'i, 's, Tree> for Inner<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        {
            self.0
                .ops(self.1, self.2)
                .into_iter()
                .map(|op| Op(self.0, op))
                .filter_map(|op| op.parse(i).ok())
                .into_iter()
                .min_by_key(|(rest, _)| rest.len())
                .ok_or_else(|| ParseError::UnexpectedToken(i.first().unwrap_or(&"").to_string()))
        }
    }
}

#[derive(Debug)]
struct Precs<'a>(&'a PrecTable, Vec<usize>);

impl<'i, 'a, 's> Parser<'i, 's, Tree> for Precs<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        self.1
            .iter()
            .map(|&p| (self.0, p))
            .filter_map(|p| {
                [
                    Closed(p.0, p.1).parse(i),
                    InfixNonAssoc(p.0, p.1).parse(i),
                    PreRight(p.0, p.1).parse(i),
                    PostLeft(p.0, p.1).parse(i),
                ]
                .into_iter()
                .filter_map(|p| p.ok())
                .into_iter()
                .min_by_key(|(rest, _)| rest.len())
                .ok_or_else(|| ParseError::UnexpectedToken(i.first().unwrap_or(&"").to_string()))
                .ok()
            })
            .into_iter()
            .min_by_key(|(rest, _)| rest.len())
            .ok_or_else(|| ParseError::UnexpectedToken(i.first().unwrap_or(&"").to_string()))
    }
}

struct InfixNonAssoc<'a>(&'a PrecTable, usize);

impl<'a, 'i, 's> Parser<'i, 's, Tree> for InfixNonAssoc<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        let succ = Precs(self.0, self.0.succ(self.1));
        let (i, left) = succ.parse(i)?;
        let (i, mut expr) = Inner(self.0, self.1, Fixity::Infix(Associativity::None)).parse(i)?;
        let (i, right) = succ.parse(i)?;
        expr.input.insert(0, left);
        expr.input.push(right);
        Ok((i, expr))
    }
}

struct Closed<'a>(&'a PrecTable, usize);

impl<'a, 'i, 's> Parser<'i, 's, Tree> for Closed<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        Inner(self.0, self.1, Fixity::Closed).parse(i)
    }
}

struct PreRight<'a>(&'a PrecTable, usize);

impl<'a, 'i, 's> Parser<'i, 's, Tree> for PreRight<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        let succ = Precs(self.0, self.0.succ(self.1));
        let pre = Or(
            Inner(self.0, self.1, Fixity::Prefix),
            (
                &succ,
                Inner(self.0, self.1, Fixity::Infix(Associativity::Right)),
            ),
        );
        let (rest, inners) = Many1(pre).parse(i)?;
        let (rest, succ) = succ.parse(rest)?;
        let mut expr = inners
            .into_iter()
            .map(|e| {
                e.map_right(|(first, mut rest)| {
                    rest.input.insert(0, first);
                    rest
                })
                .into_inner()
            })
            .rev()
            .reduce(|right, mut left| {
                left.input.push(right);
                left
            })
            .unwrap();

        expr.input.push(succ);
        Ok((rest, expr))
    }
}

struct PostLeft<'a>(&'a PrecTable, usize);

impl<'a, 'i, 's> Parser<'i, 's, Tree> for PostLeft<'a> {
    fn parse(&self, i: &'i [&'s str]) -> ParserResult<&'i [&'s str], Tree, ParseError> {
        let succ = Precs(self.0, self.0.succ(self.1));
        let post = Or(
            Inner(self.0, self.1, Fixity::Postfix),
            (
                Inner(self.0, self.1, Fixity::Infix(Associativity::Left)),
                &succ,
            ),
        );
        let (rest, succ) = succ.parse(i)?;
        let (rest, inners) = Many1(post).parse(rest)?;
        let mut expr = inners
            .into_iter()
            .map(|e| {
                e.map_right(|(mut rest, last)| {
                    rest.input.push(last);
                    rest
                })
                .into_inner()
            })
            .rev()
            .reduce(|left, mut right| {
                right.input.insert(0, left);
                right
            })
            .unwrap();

        expr.input.insert(0, succ);
        Ok((rest, expr))
    }
}

pub fn parse<'a>(graph: &'a PrecTable, input: &'a str) -> Result<Tree, ParseError> {
    let input = input
        .trim()
        .split(" ")
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>();
    let (rest, expr) = Expr(graph).parse(&input)?;
    if rest.is_empty() {
        Ok(expr)
    } else {
        Err(ParseError::UnparsedInput(rest.join(" ")))
    }
}

pub fn make_graph() -> PrecTable {
    let mut table = PrecTable::new();
    let parenthesis = Operator::new(Fixity::Closed, "(_)");
    let b = Operator::new(Fixity::Closed, "b");
    let n = Operator::new(Fixity::Closed, "n");
    let add = Operator::new(Fixity::Infix(Associativity::Left), "_+_");
    let sub = Operator::new(Fixity::Infix(Associativity::Left), "_-_");
    let eq = Operator::new(Fixity::Infix(Associativity::Left), "_==_");
    let land = Operator::new(Fixity::Infix(Associativity::Left), "_&&_");
    let fact = Operator::new(Fixity::Postfix, "_!");
    let if_then_else = Operator::new(Fixity::Prefix, "if_then_else_");
    let semicolon = Operator::new(Fixity::Infix(Associativity::Left), "_;_");
    let tuple_2 = Operator::new(Fixity::Closed, "(_,_)");
    let tuple_3 = Operator::new(Fixity::Closed, "(_,_,_)");

    table.add(&[
        (&if_then_else, 1),
        (&semicolon, 1),
        (&parenthesis, 1),
        (&tuple_2, 1),
        (&tuple_3, 1),
        (&b, 3),
        (&n, 3),
        (&fact, 9),
        (&add, 10),
        (&sub, 10),
        (&eq, 20),
        (&land, 30),
    ]);

    table
}

struct TokenStream<'i, 's> {
    tokens: &'i [&'s str],
    pos: usize,
}

impl<'i, 's> TokenStream<'i, 's> {
    pub fn new(tokens: &'i [&'s str]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> Option<&'s str> {
        self.tokens.get(self.pos).cloned()
    }

    pub fn next(&mut self) -> Option<&'s str> {
        self.pos += 1;
        self.peek()
    }

    pub fn expect(&mut self, value: &str) -> &'s str {
        let tok = self.next();
        match self.next() {
            None => panic!("Expected {value} but found 'EOF'"),
            Some(v) if v == value => v,
            Some(v) => panic!("Expected {value} but found {v}"),
        }
    }
}

struct Node<'s> {
    operator: Option<Box<Node<'s>>>,
    parts: Vec<Either<Node<'s>, &'s str>>,
}

impl<'s> Node<'s> {
    pub fn new(operator: Option<Self>, parts: Vec<Either<Self, &'s str>>) -> Self {
        Self {
            operator: operator.map(Box::new),
            parts,
        }
    }
}

struct Opp {
    parts: Vec<Option<&'static str>>,
    precedence : usize,
    associativity: Associativity
}


impl std::fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts = self
            .parts
            .iter()
            .map(|part| match part {
                Either::Left(l) => l.to_string(),
                Either::Right(r) => r.to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");

        match &self.operator {
            Some(op) => write!(
                f,
                "{}: {parts}",
                op.parts
                    .iter()
                    .map(|part| match part {
                        Either::Left(l) => l.to_string(),
                        Either::Right(r) => r.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            None => write!(f, "Primary({})", parts),
        }
    }
}

pub fn parse_expression(ts: &mut TokenStream, min_prec: usize) {
    // First, parse a primary expression or a prefix operator.
    let left = parse_primary(ts);

    // Then try to apply any operator that might continue the expression.
    loop {
        let op = try_match_operator(ts);

        if op.is_none() || op.precedence < min_prec {
            break;
        }

        let op = op.unwrap();

        // We have a candidate operator. Since our operators are mixfix, the pattern might be:
        //    [None, literal, None, literal, ..., None]
        // Here, we assume that the first hole is already filled by 'left'.
        left = apply_operator(ts, op, left)
    }

    left
}

/// Parse a primary expression. This could be a literal, variable, parenthesized expression, etc.
fn parse_primary<'i, 's>(ts: &mut TokenStream<'i, 's>) -> Node<'s> {
    let token = ts.next().expect("Expected token");
    Node::new(None, vec![Either::Right(token)])
}

const OPERATORS : [Node; 0] = [];

    /// Look ahead at the token stream to see if any operator pattern (which starts with a literal)
    /// matches. (For mixfix operators you might have operators that begin with a literal (prefix)
    /// or even with a hole. Here, for the purpose of demonstration, we assume that in an infix
    /// or mixfix operator the first part is always already provided by the preceding expression.)
fn try_match_operator<'i, 's>(ts: &mut TokenStream<'i, 's>) -> Option<()> {
    // We look ahead only at the next token (or tokens) to see if any operator literal occurs.
    // In a more complete implementation you might scan all OPERATORS.
    for op in OPERATORS {
        // We are only interested in operators that do not start with a hole.
        if let Some(parts) = op.parts[0] {
            // Check if the upcoming token matches the first literal of the operator.
            let tok = ts.peek()
            if tok and tok.value == op.parts[0]:
                return op
        }
        // Also, if the operator’s first part is None (meaning it is infix, and the left-hand side
        // already filled the first placeholder), then we need another mechanism. For now, we assume
        // that if no literal is present then the operator might have been already in progress.
    }

    None
}
//
// def apply_operator(ts, op, left):
//     """
//     Given that we have already parsed an expression for the first hole, and we have matched an
//     operator (whose pattern is e.g. [None, "if", None, "then", None, "else", None]),
//     complete the match.
//     """
//     # Determine the index of the pattern parts.
//     # If the first part of op.parts is None, then the first hole is the already-parsed left.
//     # Otherwise, if the first part is a literal, we must consume it.
//     parts = []
//     idx = 0
//     if op.parts[0] is None:
//         parts.append(left)
//         idx = 1
//     else:
//         # Consume the literal. (This branch is useful for a prefix operator.)
//         literal = ts.next()
//         if literal.value != op.parts[0]:
//             raise Exception("Operator literal mismatch")
//         parts.append(literal.value)
//         idx = 1
//
//     # Now, for each remaining part in the operator’s pattern, do one of two things:
//     # - If it is a literal (non-None), then expect that literal in the token stream.
//     # - If it is None (a placeholder), then parse an expression.
//     while idx < len(op.parts):
//         part = op.parts[idx]
//         if part is not None:
//             # Expect a literal.
//             ts.expect(part)
//             parts.append(part)
//         else:
//             # For a placeholder, we parse an expression.
//             # Use the operator's precedence for the binding power.
//             expr = parse_expression(ts, op.precedence)
//             parts.append(expr)
//         idx += 1
//     # Construct an AST node for this operator application.
//     return Node(operator=op, parts=parts)
//
// # For demonstration, let’s assume our token stream is as follows:
// # Example:  if x then y else z + 5
// tokens = [
//     Token("keyword", "if"),
//     Token("identifier", "x"),
//     Token("keyword", "then"),
//     Token("identifier", "y"),
//     Token("keyword", "else"),
//     Token("identifier", "z"),
//     Token("operator", "+"),
//     Token("number", "5")
// ]
//
// ts = TokenStream(tokens)
// ast = parse_expression(ts)
// print(ast)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combinator() {
        let i = "ab  b b a ab ab"
            .trim()
            .split(" ")
            .filter(|n| !n.is_empty())
            .collect::<Vec<_>>();

        let (rest, (ab, b)) = (Tag("ab".to_string()), Tag("b".to_string()))
            .parse(&i)
            .unwrap();

        assert_eq!(rest, &["b", "a", "ab", "ab"]);
        assert_eq!(ab, "ab");
        assert_eq!(b, "b");
    }

    #[test]
    fn testing() {
        let graph = make_graph();
        // let i = r#"if b then n && n else ( n ! , b , n == b && n ! )"#;
        let i = r#"if b then ( n && n ) else ( ( n ! , n == n , n ) )"#;
        let tree = parse(&graph, i);
        if let Err(ParseError::UnexpectedToken(m)) = tree {
            panic!("{m}");
        }
        println!("{}", tree.unwrap());
    }

    #[test]
    fn test_parser() {
        let graph = make_graph();
        let i = r#"if ( b , ( n , n , n ) ) && n + n == n ! then n else ( n + n ) ; n + n ; if b then b else n"#;
        let tree = parse(&graph, i);
        if let Err(ParseError::UnexpectedToken(m)) = tree {
            panic!("{m}");
        }
        println!("{}", tree.unwrap());
    }

    #[test]
    fn test_parser_2() {
        let graph = make_graph();
        println!("{:#?}", graph);
        let i = r#"if ( b , ( n , n , n ) ) && n + n == n ! then n else ( n + n ) ; n + n ; if b then b else n"#;
        let tree = parse(&graph, i);
        if let Err(ParseError::UnexpectedToken(m)) = tree {
            panic!("{m}");
        }
        println!("{}", tree.unwrap());
    }

    #[test]
    fn test_parser_3() {
        let graph = make_graph();
        println!("{:#?}", graph);
        let i = r#"b + b"#;
        let tree = parse(&graph, i);
        if let Err(ParseError::UnexpectedToken(m)) = tree {
            panic!("{m}");
        }
        panic!("OK: {}", tree.unwrap());
    }
}
