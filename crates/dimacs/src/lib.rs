//! This module provides parsers for the DIMACS CNF file format. Given that DIMACS files
//! can be very large, the implementation is designed to read the file in chunks. The parser also
//! will not allocate for every encountered clause, but rather re-use its buffers.
//!
//! It should be noted that the parser should not be used as a DIMACS validator. Even though it
//! should only accept valid DIMACS files, the errors are not extremely detailed. Perhaps this
//! could change over time, however.
use std::{
    io::{BufRead, BufReader, Read},
    num::NonZeroI32,
    str::FromStr,
};

use thiserror::Error;

/// A dimacs sink stores a set of clauses and allows for new variables to be created.
pub trait DimacsSink {
    /// Add a new clause to the formula.
    fn add_clause(&mut self, clause: &[NonZeroI32]);
}

#[derive(Debug, Error)]
pub enum DimacsParseError {
    #[error("failed to read file")]
    Io(#[from] std::io::Error),

    #[error("missing dimacs header")]
    MissingHeader,

    #[error("'{0}' is an invalid header")]
    InvalidHeader(String),

    #[error("multiple dimacs headers found")]
    DuplicateHeader,

    #[error("unexpected character '{0}'")]
    UnexpectedCharacter(char),

    #[error("'{0}' is an invalid DIMACS literal")]
    InvalidLiteral(String),

    #[error("the last clause in the source is not terminated with a '0'")]
    UnterminatedClause,

    #[error("expected to parse {expected} clauses, but parsed {parsed}")]
    IncorrectClauseCount { expected: usize, parsed: usize },
}

pub fn parse_cnf<Sink: DimacsSink>(
    source: impl Read,
    sink_factory: impl FnOnce(&CNFHeader) -> Sink,
) -> Result<Sink, DimacsParseError> {
    let mut reader = BufReader::new(source);
    let mut parser = DimacsParser::<Sink, _>::new(sink_factory);

    loop {
        let num_bytes = {
            let data = reader.fill_buf()?;

            if data.is_empty() {
                return parser.complete();
            }

            parser.parse_chunk(data)?;
            data.len()
        };

        reader.consume(num_bytes);
    }
}

struct DimacsParser<Sink: DimacsSink, SinkConstructor> {
    sink_constructor: Option<SinkConstructor>,
    sink: Option<Sink>,
    header: Option<CNFHeader>,
    buffer: String,
    clause: Vec<NonZeroI32>,
    state: ParseState,
    parsed_clauses: usize,
}

enum ParseState {
    StartLine,
    Header,
    Comment,
    Literal,
    NegativeLiteral,
    Clause,
}

impl<Sink, SinkConstructor> DimacsParser<Sink, SinkConstructor>
where
    Sink: DimacsSink,
    SinkConstructor: FnOnce(&CNFHeader) -> Sink,
{
    /// Construct a new DIMACS parser based on the sink constructor arguments and the callback to
    /// be executed when a clause is completely parsed.
    fn new(sink_constructor: SinkConstructor) -> Self {
        DimacsParser {
            sink_constructor: Some(sink_constructor),
            sink: None,
            header: None,
            buffer: String::new(),
            clause: vec![],
            state: ParseState::StartLine,
            parsed_clauses: 0,
        }
    }

    /// Parse the next chunk of bytes. This may start in the middle of parsing a clause or file
    /// header, and may end in such a state as well.
    fn parse_chunk(&mut self, chunk: &[u8]) -> Result<(), DimacsParseError> {
        for byte in chunk {
            match self.state {
                ParseState::StartLine => match byte {
                    b if b.is_ascii_whitespace() => {} // Continue consuming whitespace.

                    b'p' => {
                        self.state = ParseState::Header;
                        self.buffer.clear();
                        self.buffer.push('p');
                    }

                    b'c' => {
                        self.state = ParseState::Comment;
                    }

                    b @ b'1'..=b'9' => {
                        self.start_literal(b, true);
                    }

                    //covers the exotic case of having an empty clause in the dimacs file
                    b'0' => self.finish_clause()?,

                    b'-' => self.start_literal(&b'-', false),

                    b => return Err(DimacsParseError::UnexpectedCharacter(*b as char)),
                },

                ParseState::Header => match byte {
                    b'\n' => {
                        self.init_formula()?;
                        self.state = ParseState::StartLine;
                    }

                    b => self.buffer.push(*b as char),
                },

                ParseState::Comment => {
                    // Ignore all other bytes until we find a new-line, at which point the comment
                    // ends.
                    if *byte == b'\n' {
                        self.state = ParseState::StartLine;
                    }
                }

                ParseState::Literal => match byte {
                    b if b.is_ascii_whitespace() => {
                        self.finish_literal()?;
                    }

                    b @ b'0'..=b'9' => self.buffer.push(*b as char),

                    b => return Err(DimacsParseError::UnexpectedCharacter(*b as char)),
                },

                ParseState::NegativeLiteral => match byte {
                    b @ b'1'..=b'9' => {
                        self.buffer.push(*b as char);
                        self.state = ParseState::Literal;
                    }

                    b => return Err(DimacsParseError::UnexpectedCharacter(*b as char)),
                },

                ParseState::Clause => match byte {
                    b'0' => self.finish_clause()?,

                    // When a new-line is encountered, it does not mean the clause is terminated.
                    // We switch to the StartLine state to handle comments and leading whitespace.
                    // However, the clause buffer is not cleared so the clause that is being parsed
                    // is kept in-memory and will continue to be parsed as soon as a literal is
                    // encountered.
                    b'\n' => self.state = ParseState::StartLine,
                    b if b.is_ascii_whitespace() => {} // Ignore whitespace.

                    b @ b'1'..=b'9' => self.start_literal(b, true),
                    b'-' => self.start_literal(&b'-', false),

                    b => return Err(DimacsParseError::UnexpectedCharacter(*b as char)),
                },
            }
        }

        Ok(())
    }

    fn start_literal(&mut self, b: &u8, is_positive: bool) {
        self.state = if is_positive {
            ParseState::Literal
        } else {
            ParseState::NegativeLiteral
        };

        self.buffer.clear();
        self.buffer.push(*b as char);
    }

    fn complete(self) -> Result<Sink, DimacsParseError> {
        let sink = self.sink.ok_or(DimacsParseError::MissingHeader)?;
        let header = self
            .header
            .expect("if sink is present then header is present");

        if !self.clause.is_empty() {
            Err(DimacsParseError::UnterminatedClause)
        } else if header.num_clauses != self.parsed_clauses {
            Err(DimacsParseError::IncorrectClauseCount {
                expected: header.num_clauses,
                parsed: self.parsed_clauses,
            })
        } else {
            Ok(sink)
        }
    }

    fn init_formula(&mut self) -> Result<(), DimacsParseError> {
        let header = self.buffer.trim().parse::<CNFHeader>()?;
        let sink_constructor = self
            .sink_constructor
            .take()
            .expect("only parse header once");

        self.sink = Some(sink_constructor(&header));

        self.header = Some(header);

        Ok(())
    }

    fn finish_literal(&mut self) -> Result<(), DimacsParseError> {
        let dimacs_code = self
            .buffer
            .parse::<i32>()
            .map_err(|_| DimacsParseError::InvalidLiteral(self.buffer.clone()))?;

        let literal = NonZeroI32::new(dimacs_code).expect("cannot be 0 here");
        self.clause.push(literal);
        self.state = ParseState::Clause;

        Ok(())
    }

    fn finish_clause(&mut self) -> Result<(), DimacsParseError> {
        let sink = self.sink.as_mut().ok_or(DimacsParseError::MissingHeader)?;
        self.parsed_clauses += 1;
        sink.add_clause(&self.clause);
        self.clause.clear();

        Ok(())
    }
}

pub struct CNFHeader {
    pub num_variables: usize,
    pub num_clauses: usize,
}

impl FromStr for CNFHeader {
    type Err = DimacsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("p cnf ") {
            return Err(DimacsParseError::InvalidHeader(s.to_string()));
        }

        let mut components = s.trim().split(' ').skip(2);

        let num_variables = next_header_component::<usize>(&mut components, s)?;
        let num_clauses = next_header_component::<usize>(&mut components, s)?;

        if components.next().is_some() {
            return Err(DimacsParseError::InvalidHeader(s.to_string()));
        }

        Ok(Self {
            num_variables,
            num_clauses,
        })
    }
}

fn next_header_component<'a, Num: FromStr>(
    components: &mut impl Iterator<Item = &'a str>,
    header: &str,
) -> Result<Num, DimacsParseError> {
    components
        .next()
        .ok_or_else(|| DimacsParseError::InvalidHeader(header.to_string()))?
        .parse::<Num>()
        .map_err(|_| DimacsParseError::InvalidHeader(header.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_instance_is_read() {
        let source = "p cnf 2 2\n1 -2 0\n-1 2 0";
        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2], vec![-1, 2]], formula);
    }

    #[test]
    fn instance_with_two_character_codes_is_accepted() {
        let source = "p cnf 11 2\n1 -2 10 0\n-1 2 -11 0";
        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2, 10], vec![-1, 2, -11]], formula);
    }

    #[test]
    fn trailing_whitespace_is_ignored() {
        let source = "p cnf 2 2\n1 -2 0\n-1 2 0\n";
        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2], vec![-1, 2]], formula);
    }

    #[test]
    fn comments_are_ignored() {
        let source = "c this is\nc a comment\np cnf 2 2\n1 -2 0\nc within the file\n-1 2 0\n";
        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2], vec![-1, 2]], formula);
    }

    #[test]
    fn whitespace_is_ignored() {
        let source = r#"
            p cnf 2 2
             1 -2 0
            -1  2 0
        "#;

        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2], vec![-1, 2]], formula);
    }

    #[test]
    fn empty_lines_are_ignored() {
        let source = r#"

            p cnf 2 2


             1 -2 0

            -1  2 0
        "#;

        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2], vec![-1, 2]], formula);
    }

    #[test]
    fn clauses_on_same_line_are_separated() {
        let source = "p cnf 2 2\n1 -2 0 -1 2 0";
        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2], vec![-1, 2]], formula);
    }

    #[test]
    fn new_lines_do_not_terminate_clause() {
        let source = "p cnf 2 2\n1\n-2 0 -1 2\n 0";
        let formula = parse_cnf_source(source);

        assert_eq!(vec![vec![1, -2], vec![-1, 2]], formula);
    }

    #[test]
    fn negative_zero_is_an_unexpected_sequence() {
        let source = "p cnf 2 1\n1 -2 -0";
        let err = get_cnf_parse_error(source);

        assert!(matches!(err, DimacsParseError::UnexpectedCharacter('0')));
    }

    #[test]
    fn incomplete_clause_causes_error() {
        let source = "p cnf 2 1\n1 -2";
        let err = get_cnf_parse_error(source);

        assert!(matches!(err, DimacsParseError::UnterminatedClause));
    }

    #[test]
    fn incorrect_reported_clause_count() {
        let source = "p cnf 2 2\n1 -2 0";
        let err = get_cnf_parse_error(source);

        assert!(matches!(
            err,
            DimacsParseError::IncorrectClauseCount {
                expected: 2,
                parsed: 1
            }
        ));
    }

    fn parse_cnf_source(source: &str) -> Vec<Vec<i32>> {
        parse_cnf::<Vec<Vec<i32>>>(source.as_bytes(), |_| vec![]).expect("valid dimacs")
    }

    fn get_cnf_parse_error(source: &str) -> DimacsParseError {
        parse_cnf::<Vec<Vec<i32>>>(source.as_bytes(), |_| vec![]).expect_err("invalid dimacs")
    }

    impl DimacsSink for Vec<Vec<i32>> {
        fn add_clause(&mut self, clause: &[NonZeroI32]) {
            self.push(clause.iter().map(|lit| lit.get()).collect());
        }
    }
}
