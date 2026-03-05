//! Тесты для LL(1) анализа грамматики

#[cfg(test)]
mod tests {
    use minic::parser::ll1::{FirstFollowCalculator, GrammarSymbol, Production};

    #[test]
    fn test_expression_grammar_ll1() {
        let productions = vec![
            Production {
                left: "expression".to_string(),
                right: vec![GrammarSymbol::NonTerminal("assignment".to_string())],
            },
            Production {
                left: "assignment".to_string(),
                right: vec![
                    GrammarSymbol::NonTerminal("logical_or".to_string()),
                    GrammarSymbol::NonTerminal("assignment_tail".to_string()),
                ],
            },
            Production {
                left: "assignment_tail".to_string(),
                right: vec![
                    GrammarSymbol::Terminal("=".to_string()),
                    GrammarSymbol::NonTerminal("assignment".to_string()),
                ],
            },
            Production {
                left: "assignment_tail".to_string(),
                right: vec![GrammarSymbol::Epsilon],
            },
        ];

        let mut calculator = FirstFollowCalculator::new(productions);

        calculator.compute_first();
        let first = calculator.first_sets().clone();

        calculator.compute_follow();
        let follow = calculator.follow_sets().clone();

        println!("First множества: {:?}", first);
        println!("Follow множества: {:?}", follow);

        assert!(calculator.is_ll1());
    }

    #[test]
    fn test_classical_grammar() {
        let productions = vec![
            Production {
                left: "E".to_string(),
                right: vec![
                    GrammarSymbol::NonTerminal("T".to_string()),
                    GrammarSymbol::NonTerminal("E'".to_string()),
                ],
            },
            Production {
                left: "E'".to_string(),
                right: vec![
                    GrammarSymbol::Terminal("+".to_string()),
                    GrammarSymbol::NonTerminal("T".to_string()),
                    GrammarSymbol::NonTerminal("E'".to_string()),
                ],
            },
            Production {
                left: "E'".to_string(),
                right: vec![GrammarSymbol::Epsilon],
            },
            Production {
                left: "T".to_string(),
                right: vec![
                    GrammarSymbol::NonTerminal("F".to_string()),
                    GrammarSymbol::NonTerminal("T'".to_string()),
                ],
            },
            Production {
                left: "T'".to_string(),
                right: vec![
                    GrammarSymbol::Terminal("*".to_string()),
                    GrammarSymbol::NonTerminal("F".to_string()),
                    GrammarSymbol::NonTerminal("T'".to_string()),
                ],
            },
            Production {
                left: "T'".to_string(),
                right: vec![GrammarSymbol::Epsilon],
            },
            Production {
                left: "F".to_string(),
                right: vec![GrammarSymbol::Terminal("id".to_string())],
            },
            Production {
                left: "F".to_string(),
                right: vec![
                    GrammarSymbol::Terminal("(".to_string()),
                    GrammarSymbol::NonTerminal("E".to_string()),
                    GrammarSymbol::Terminal(")".to_string()),
                ],
            },
        ];

        let mut calculator = FirstFollowCalculator::new(productions);

        calculator.compute_first();
        let first = calculator.first_sets().clone();

        calculator.compute_follow();
        let follow = calculator.follow_sets().clone();

        println!("First множества: {:#?}", first);
        println!("Follow множества: {:#?}", follow);

        assert!(calculator.is_ll1());
    }
}
