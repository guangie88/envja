#[derive(Parser)]
#[grammar = "envja.pest"]
pub struct EnvjaParser;

#[cfg(test)]
mod tests {
    use crate::{EnvjaParser, Rule};
    use pest::Parser;

    #[test]
    fn expr() {
        const TEMPLATE: &str = r#"{{_}}"#;
        let _ = EnvjaParser::parse(Rule::doc, TEMPLATE)
            .unwrap_or_else(|e| panic!("{}", e));
    }

    #[test]
    fn expr_underscore_between() {
        const TEMPLATE: &str = r#" {{ FOO_BAR }} "#;
        let _ = EnvjaParser::parse(Rule::doc, TEMPLATE)
            .unwrap_or_else(|e| panic!("{}", e));
    }

    #[test]
    fn text_cpp() {
        const TEMPLATE: &str = r#"
int main() {
    return 0;
}
"#;

        let _ = EnvjaParser::parse(Rule::doc, TEMPLATE)
            .unwrap_or_else(|e| panic!("{}", e));
    }

    #[test]
    fn text_cpp_template() {
        const TEMPLATE: &str = r#"
{% if FOO %}#include <{{FOO}}>{% endif %}
int main() {
    return {{ BAR }}
}
"#;

        let _ = EnvjaParser::parse(Rule::doc, TEMPLATE)
            .unwrap_or_else(|e| panic!("{}", e));
    }
}
