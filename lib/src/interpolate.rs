use crate::{EnvjaParser, Rule};
use pest::{iterators::Pair, Parser};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub type Mappings = HashMap<String, String>;
pub type DynError = Box<dyn Error>;

#[derive(Clone, Debug)]
pub struct InterpolateError {
    rule: Rule,
    msg: String,
}

impl InterpolateError {
    pub fn new<S: Into<String>>(rule: Rule, msg: S) -> InterpolateError {
        InterpolateError {
            rule,
            msg: msg.into(),
        }
    }
}

impl fmt::Display for InterpolateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.rule)
    }
}

impl Error for InterpolateError {}

fn interpolate_if_stmt_impl(
    pair: Pair<'_, Rule>,
    buffer: &mut String,
    mappings: &Mappings,
) -> Result<(), DynError> {
    let mut will_interpolate = false;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::if_stmt_start => {
                for inner_pair in p.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::ident => {
                            let identifier = inner_pair.as_str();
                            let replacement = mappings.get(identifier);

                            match replacement {
                                Some(rep) => {
                                    will_interpolate = !rep.is_empty();
                                }
                                None => {
                                    will_interpolate = false;
                                }
                            }
                        }
                        r => {
                            Err(InterpolateError::new(
                                r,
                                "Unexpected sub-rule for if_stmt_start found",
                            ))?;
                        }
                    }
                }
            }
            Rule::if_stmt_end => {
                // do nothing
            }
            Rule::compound => {
                if will_interpolate {
                    interpolate_compound_impl(p, buffer, mappings)?;
                }
            }
            r => {
                Err(InterpolateError::new(
                    r,
                    "Unexpected sub-rule for if_stmt found",
                ))?;
            }
        }
    }

    Ok(())
}

fn interpolate_stmt_impl(
    pair: Pair<'_, Rule>,
    buffer: &mut String,
    mappings: &Mappings,
) -> Result<(), DynError> {
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::if_stmt => {
                interpolate_if_stmt_impl(p, buffer, mappings)?;
            }
            r => {
                Err(InterpolateError::new(
                    r,
                    "Unexpected sub-rule for stmt found",
                ))?;
            }
        }
    }

    Ok(())
}

fn interpolate_expr_impl(
    pair: Pair<'_, Rule>,
    buffer: &mut String,
    mappings: &Mappings,
) -> Result<(), DynError> {
    for p in pair.into_inner() {
        match p.as_rule() {
            r @ Rule::ident => {
                let key = p.as_str();
                let val = mappings.get(key).ok_or_else(|| {
                    InterpolateError::new(
                        r,
                        format!("No key '{}' found for interpolation", key),
                    )
                })?;
                buffer.push_str(val);
            }
            r => {
                Err(InterpolateError::new(
                    r,
                    "Unexpected sub-rule for expr found",
                ))?;
            }
        }
    }

    Ok(())
}

fn interpolate_compound_impl(
    pair: Pair<'_, Rule>,
    buffer: &mut String,
    mappings: &Mappings,
) -> Result<(), DynError> {
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::stmt => {
                interpolate_stmt_impl(p, buffer, mappings)?;
            }
            Rule::expr => {
                interpolate_expr_impl(p, buffer, mappings)?;
            }
            Rule::comm => {
                // Do nothing
            }
            Rule::text => {
                buffer.push_str(p.as_str());
            }
            Rule::compound => {
                interpolate_compound_impl(p, buffer, mappings)?;
            }
            r => {
                Err(InterpolateError::new(
                    r,
                    "Unexpected sub-rule at compound found",
                ))?;
            }
        }
    }

    Ok(())
}

pub fn interpolate(
    template: &str,
    mappings: &Mappings,
) -> Result<String, DynError> {
    let pairs = EnvjaParser::parse(Rule::doc, template)?;
    let mut buffer = String::new();

    for p in pairs {
        match p.as_rule() {
            Rule::compound => {
                interpolate_compound_impl(p, &mut buffer, mappings)?;
            }
            Rule::EOI => {
                // do nothing
            }
            r => {
                Err(InterpolateError::new(
                    r,
                    "Unexpected sub-rule at top-level found",
                ))?;
            }
        }
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_nothing() {
        assert_eq!(interpolate("", &Mappings::new()).unwrap(), "");
    }

    #[test]
    fn interpolate_simple_if_stmt() {
        const TEMPLATE: &str = r#"xxx{% if HAS_VAL %}yyy{% endif %}zzz"#;
        const EXPECTED_NO_MAPPING: &str = r#"xxxzzz"#;
        const EXPECTED: &str = r#"xxxyyyzzz"#;

        assert_eq!(
            interpolate(TEMPLATE, &Mappings::new()).unwrap(),
            EXPECTED_NO_MAPPING
        );

        let mappings = {
            let mut m = Mappings::new();
            m.insert("HAS_VAL".to_owned(), "true".to_owned());
            m
        };

        assert_eq!(interpolate(TEMPLATE, &mappings).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_if_stmt_with_expr_within() {
        const TEMPLATE: &str = r#"{% if VAL %}VAL={{VAL}}{% endif %}"#;
        const EXPECTED_NO_MAPPING: &str = r#""#;
        const EXPECTED: &str = r#"VAL=hello"#;

        assert_eq!(
            interpolate(TEMPLATE, &Mappings::new()).unwrap(),
            EXPECTED_NO_MAPPING
        );

        let mappings = {
            let mut m = Mappings::new();
            m.insert("VAL".to_owned(), "hello".to_owned());
            m
        };

        assert_eq!(interpolate(TEMPLATE, &mappings).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_if_stmt_with_expr_within_multiline() {
        const TEMPLATE: &str = r#"{% if VAL %}
VAL={{VAL}}
{% endif %}"#;

        const EXPECTED_NO_MAPPING: &str = r#""#;

        const EXPECTED: &str = r#"
VAL=This is a long string
"#;

        assert_eq!(
            interpolate(TEMPLATE, &Mappings::new()).unwrap(),
            EXPECTED_NO_MAPPING
        );

        let mappings = {
            let mut m = Mappings::new();
            m.insert("VAL".to_owned(), "This is a long string".to_owned());
            m
        };

        assert_eq!(interpolate(TEMPLATE, &mappings).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_cpp_linux() {
        const TEMPLATE: &str = r#"
{% if LINUX_HEADER %}#include <{{LINUX_HEADER}}>{% endif %}
int main() {
    return {{ RET }};
}
"#;

        const EXPECTED: &str = r#"
#include <unistd.h>
int main() {
    return 123;
}
"#;

        assert!(interpolate(TEMPLATE, &Mappings::new()).is_err());

        let mappings = {
            let mut m = Mappings::new();
            m.insert("LINUX_HEADER".to_owned(), "unistd.h".to_owned());
            m.insert("RET".to_owned(), "123".to_owned());
            m
        };

        assert_eq!(interpolate(TEMPLATE, &mappings).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_empty_comment_line() {
        const TEMPLATE: &str = r#"{##}"#;
        const EXPECTED: &str = r#""#;
        assert_eq!(interpolate(TEMPLATE, &Mappings::new()).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_empty_comment_block() {
        const TEMPLATE: &str = r#"{#
#}"#;
        const EXPECTED: &str = r#""#;
        assert_eq!(interpolate(TEMPLATE, &Mappings::new()).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_simple_comment_line() {
        const TEMPLATE: &str = r#"{# This is a comment line #}"#;
        const EXPECTED: &str = r#""#;
        assert_eq!(interpolate(TEMPLATE, &Mappings::new()).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_simple_comment_block() {
        const TEMPLATE: &str = r#"{# This
is a comment
block
#}"#;
        const EXPECTED: &str = r#""#;
        assert_eq!(interpolate(TEMPLATE, &Mappings::new()).unwrap(), EXPECTED);
    }

    #[test]
    fn interpolate_comment_with_mixed() {
        const TEMPLATE: &str =
            r#"{% if SHOW %}{# Comment1 #}How{#Comment2#}are{{SHOW}}{%endif%}"#;
        const EXPECTED_NO_MAPPING: &str = r#""#;
        const EXPECTED: &str = r#"Howareyou"#;

        assert_eq!(
            interpolate(TEMPLATE, &Mappings::new()).unwrap(),
            EXPECTED_NO_MAPPING
        );

        let mappings = {
            let mut m = Mappings::new();
            m.insert("SHOW".to_owned(), "you".to_owned());
            m
        };

        assert_eq!(interpolate(TEMPLATE, &mappings).unwrap(), EXPECTED);
    }
}
