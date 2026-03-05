//! LL(1) анализ грамматики
//!
//! Этот модуль предоставляет инструменты для проверки,
//! является ли грамматика LL(1), и вычисления First/Follow множеств.

use std::collections::{HashMap, HashSet};

/// Тип символа в грамматике
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GrammarSymbol {
    Terminal(String),
    NonTerminal(String),
    Epsilon,
    EndOfFile,
}

/// Правило грамматики
#[derive(Debug, Clone)]
pub struct Production {
    pub left: String,
    pub right: Vec<GrammarSymbol>,
}

/// Множества First для нетерминалов
pub type FirstSets = HashMap<String, HashSet<GrammarSymbol>>;

/// Множества Follow для нетерминалов
pub type FollowSets = HashMap<String, HashSet<GrammarSymbol>>;

/// Калькулятор First/Follow множеств
pub struct FirstFollowCalculator {
    productions: Vec<Production>,
    first_sets: FirstSets,
    follow_sets: FollowSets,
    non_terminals: HashSet<String>,
    _terminals: HashSet<String>,
}

impl FirstFollowCalculator {
    pub fn new(productions: Vec<Production>) -> Self {
        let mut non_terminals = HashSet::new();
        let mut terminals = HashSet::new();

        for prod in &productions {
            non_terminals.insert(prod.left.clone());
            for sym in &prod.right {
                match sym {
                    GrammarSymbol::Terminal(t) => {
                        terminals.insert(t.clone());
                    }
                    GrammarSymbol::NonTerminal(nt) => {
                        non_terminals.insert(nt.clone());
                    }
                    _ => {}
                }
            }
        }

        Self {
            productions,
            first_sets: HashMap::new(),
            follow_sets: HashMap::new(),
            non_terminals,
            _terminals: terminals,
        }
    }

    /// Вычисляет First множества для всех нетерминалов
    pub fn compute_first(&mut self) -> &FirstSets {
        // Инициализация
        for nt in &self.non_terminals {
            self.first_sets.insert(nt.clone(), HashSet::new());
        }

        let mut changed = true;
        while changed {
            changed = false;

            for prod in &self.productions {
                let left = &prod.left;
                let right = &prod.right;

                let current_first = self.first_sets.get(left).unwrap().clone();
                let new_first = self.first_for_sequence(right);

                if new_first.len() > current_first.len() {
                    self.first_sets.get_mut(left).unwrap().extend(new_first);
                    changed = true;
                }
            }
        }

        &self.first_sets
    }

    /// Вычисляет First для последовательности символов
    fn first_for_sequence(&self, seq: &[GrammarSymbol]) -> HashSet<GrammarSymbol> {
        let mut result = HashSet::new();

        if seq.is_empty() {
            result.insert(GrammarSymbol::Epsilon);
            return result;
        }

        for (i, sym) in seq.iter().enumerate() {
            match sym {
                GrammarSymbol::Terminal(t) => {
                    result.insert(GrammarSymbol::Terminal(t.clone()));
                    break;
                }
                GrammarSymbol::NonTerminal(nt) => {
                    if let Some(first_nt) = self.first_sets.get(nt) {
                        let mut has_epsilon = false;
                        for s in first_nt {
                            if *s == GrammarSymbol::Epsilon {
                                has_epsilon = true;
                            } else {
                                result.insert(s.clone());
                            }
                        }

                        if !has_epsilon || i == seq.len() - 1 {
                            break;
                        }
                    }
                }
                GrammarSymbol::Epsilon => {
                    result.insert(GrammarSymbol::Epsilon);
                    break;
                }
                GrammarSymbol::EndOfFile => {
                    result.insert(GrammarSymbol::EndOfFile);
                    break;
                }
            }
        }

        result
    }

    /// Возвращает First множества (для тестов)
    pub fn first_sets(&self) -> &FirstSets {
        &self.first_sets
    }

    /// Возвращает Follow множества (для тестов)
    pub fn follow_sets(&self) -> &FollowSets {
        &self.follow_sets
    }

    /// Вычисляет Follow множества для всех нетерминалов
    pub fn compute_follow(&mut self) -> &FollowSets {
        for nt in &self.non_terminals {
            self.follow_sets.insert(nt.clone(), HashSet::new());
        }

        if let Some(start) = self.non_terminals.iter().next() {
            self.follow_sets
                .get_mut(start)
                .unwrap()
                .insert(GrammarSymbol::EndOfFile);
        }

        let mut changed = true;
        while changed {
            changed = false;

            let current_follows = self.follow_sets.clone();

            for prod in &self.productions {
                let left = &prod.left;
                let right = &prod.right;

                for (i, sym) in right.iter().enumerate() {
                    if let GrammarSymbol::NonTerminal(b) = sym {
                        let beta = &right[i + 1..];
                        let first_beta = self.first_for_sequence(beta);

                        let mut new_symbols = HashSet::new();

                        for s in &first_beta {
                            if *s != GrammarSymbol::Epsilon {
                                new_symbols.insert(s.clone());
                            }
                        }

                        if first_beta.contains(&GrammarSymbol::Epsilon) || beta.is_empty() {
                            if let Some(follow_a) = current_follows.get(left) {
                                for s in follow_a {
                                    new_symbols.insert(s.clone());
                                }
                            }
                        }

                        if !new_symbols.is_empty() {
                            let follow_b = self.follow_sets.get_mut(b).unwrap();
                            let old_len = follow_b.len();
                            follow_b.extend(new_symbols);
                            if follow_b.len() > old_len {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        &self.follow_sets
    }

    /// Проверяет, является ли грамматика LL(1)
    pub fn is_ll1(&self) -> bool {
        let mut prod_map: HashMap<String, Vec<&Production>> = HashMap::new();

        for prod in &self.productions {
            prod_map
                .entry(prod.left.clone())
                .or_insert_with(Vec::new)
                .push(prod);
        }

        for (left, prods) in prod_map {
            let mut first_sets = Vec::new();

            for prod in prods {
                first_sets.push(self.first_for_sequence(&prod.right));
            }

            for i in 0..first_sets.len() {
                for j in i + 1..first_sets.len() {
                    let intersection: HashSet<_> = first_sets[i]
                        .intersection(&first_sets[j])
                        .filter(|s| **s != GrammarSymbol::Epsilon)
                        .collect();

                    if !intersection.is_empty() {
                        println!(
                            "LL(1) конфликт для {}: {:?} и {:?} пересекаются",
                            left, first_sets[i], first_sets[j]
                        );
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ll1::{FirstFollowCalculator, GrammarSymbol, Production};

    #[test]
    fn test_first_follow() {
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
        let first = calculator.first_sets.clone();

        calculator.compute_follow();
        let follow = calculator.follow_sets.clone();

        println!("First множества: {:#?}", first);
        println!("Follow множества: {:#?}", follow);

        assert!(calculator.is_ll1());
    }

    #[test]
    fn test_separate_computation() {
        let productions = vec![
            Production {
                left: "S".to_string(),
                right: vec![
                    GrammarSymbol::NonTerminal("A".to_string()),
                    GrammarSymbol::NonTerminal("B".to_string()),
                ],
            },
            Production {
                left: "A".to_string(),
                right: vec![GrammarSymbol::Terminal("a".to_string())],
            },
            Production {
                left: "A".to_string(),
                right: vec![GrammarSymbol::Epsilon],
            },
            Production {
                left: "B".to_string(),
                right: vec![GrammarSymbol::Terminal("b".to_string())],
            },
        ];

        let mut calculator = FirstFollowCalculator::new(productions);

        calculator.compute_first();
        let first_result = calculator.first_sets.clone();

        calculator.compute_follow();
        let follow_result = calculator.follow_sets.clone();

        println!("First: {:?}", first_result);
        println!("Follow: {:?}", follow_result);
    }
}
