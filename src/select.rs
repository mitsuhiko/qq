use failure::Fail;
use pest::Parser;
use pest_derive::Parser;

#[derive(Fail, Debug)]
#[fail(display = "{}", _0)]
pub struct SelectorParseError(pest::error::Error<Rule>);

impl SelectorParseError {
    /// Return the column of where the error ocurred.
    #[allow(unused)]
    pub fn column(&self) -> usize {
        match self.0.line_col {
            pest::error::LineColLocation::Pos((_, col)) => col,
            pest::error::LineColLocation::Span((_, col), _) => col,
        }
    }
}

#[derive(Parser)]
#[grammar = "select_grammar.pest"]
pub struct SelectParser;

#[derive(Debug)]
pub enum Segment {
    Key(String),
    Index(i32),
    RangeTo(i32),
    RangeFrom(i32),
    Range(i32, i32),
    FullRange,
}

#[derive(Debug)]
pub struct Selector {
    segments: Vec<Segment>,
}

pub fn debug_selector(select: &str) -> Result<Selector, SelectorParseError> {
    let pair = SelectParser::parse(Rule::selector, select)
        .map_err(SelectorParseError)?
        .next()
        .unwrap();
    let mut segments = vec![];

    for segment_pair in pair.into_inner() {
        segments.push(match segment_pair.as_rule() {
            Rule::identity => continue,
            Rule::key => Segment::Key(segment_pair.as_str()[1..].to_string()),
            Rule::subscript => {
                let subscript_rule = segment_pair.into_inner().next().unwrap();
                match subscript_rule.as_rule() {
                    Rule::int => Segment::Index(subscript_rule.as_str().parse().unwrap()),
                    Rule::string => {
                        let s = subscript_rule.as_str();
                        let mut was_backslash = false;
                        Segment::Key(
                            s[1..s.len() - 1]
                                .chars()
                                .filter_map(|c| {
                                    let rv = match c {
                                        '\\' if !was_backslash => {
                                            was_backslash = true;
                                            return None;
                                        }
                                        other => other,
                                    };
                                    was_backslash = false;
                                    Some(rv)
                                })
                                .collect(),
                        )
                    }
                    _ => unreachable!(),
                }
            }
            Rule::full_range => Segment::FullRange,
            Rule::range => {
                let mut int_rule = segment_pair
                    .into_inner()
                    .map(|x| x.as_str().parse().unwrap());
                Segment::Range(int_rule.next().unwrap(), int_rule.next().unwrap())
            }
            Rule::range_to => {
                let int_rule = segment_pair.into_inner().next().unwrap();
                Segment::RangeTo(int_rule.as_str().parse().unwrap())
            }
            Rule::range_from => {
                let int_rule = segment_pair.into_inner().next().unwrap();
                Segment::RangeFrom(int_rule.as_str().parse().unwrap())
            }
            Rule::EOI => continue,
            _ => unreachable!(),
        });
    }

    Ok(Selector { segments })
}
