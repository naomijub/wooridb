use std::str::FromStr;

use crate::{logic::parse_value, select::algebra_functions, ToSelect, Types, Wql};
use serde::{Deserialize, Serialize};

pub fn where_selector(
    entity_name: String,
    arg: ToSelect,
    chars: &mut std::str::Chars,
) -> Result<Wql, String> {
    let mut open = chars.skip_while(|c| c.is_whitespace()).take(1);

    if open.next() != Some('{') {
        return Err(String::from(
            "WHERE clauses must be contained inside ` {...}`",
        ));
    }

    let mut clauses = Vec::new();
    let mut clause = String::new();
    loop {
        match chars.next() {
            Some(',') => {
                clauses.push(clause);
                clause = String::new();
            }
            Some('}') | None => break,
            Some(c) => clause.push(c),
        }
    }

    let clauses = clauses
        .into_iter()
        .filter(|c| !c.is_empty())
        .map(|c| {
            let mut chs = c.trim().chars();
            set_clause(&entity_name, &mut chs)
        })
        .collect::<Vec<Clause>>();
    if clauses.is_empty() {
        return Err(String::from("WHERE clause cannot be empty"));
    }

    let next_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();

    Ok(Wql::SelectWhere(
        entity_name,
        arg,
        clauses,
        algebra_functions(next_symbol, chars)?,
    ))
}

fn set_clause(entity_name: &str, chs: &mut std::str::Chars) -> Clause {
    let c_str: String = chs
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c != &',')
        .collect();

    if c_str.starts_with("?*") {
        clause_entity_definition(entity_name, &c_str)
    } else if c_str.starts_with('(') && c_str.ends_with(')') {
        clause_function(entity_name, &c_str[1..c_str.len() - 1])
    } else {
        Clause::Error
    }
}
fn clause_function(entity_name: &str, clause: &str) -> Clause {
    let args: Vec<&str> = clause
        .split(' ')
        .filter(|c| !c.is_empty())
        .map(str::trim)
        .collect();
    if args.len() < 3 {
        return Clause::Error;
    }

    match &args[0].to_lowercase()[..] {
        ">=" | ">" | "==" | "<=" | "<" | "like" => {
            let mut chs = args[2].chars();
            let function = Function::from_str(args[0]).unwrap();
            if Function::Error == function {
                Clause::Error
            } else if let Ok(value) = parse_value(chs.next().unwrap(), &mut chs) {
                Clause::SimpleComparisonFunction(function, args[1].to_string(), value)
            } else {
                Clause::Error
            }
        }
        "in" | "between" => {
            let function = Function::from_str(args[0]).unwrap();
            let key = args[1].to_string();
            let values = args[2..]
                .iter()
                .filter(|s| !s.is_empty())
                .filter_map(|s| {
                    let mut chs = s.chars();
                    parse_value(chs.next().unwrap(), &mut chs).ok()
                })
                .collect::<Vec<Types>>();
            if (Function::Between == function && values.len() != 2)
                || values.iter().any(|t| t == &Types::Nil)
            {
                Clause::Error
            } else {
                Clause::ComplexComparisonFunctions(function, key, values)
            }
        }
        "or" => {
            let clauses = or_clauses(entity_name, clause);
            Clause::Or(Function::Or, clauses)
        }
        _ => Clause::Error,
    }
}

fn or_clauses(entity_name: &str, clause: &str) -> Vec<Clause> {
    let mut chars = clause[2..].chars();
    let mut clauses = Vec::new();
    let mut clause = String::new();
    loop {
        match chars.next() {
            Some(',') => {
                clauses.push(clause);
                clause = String::new();
            }
            Some(')') => {
                clause.push(')');
                clauses.push(clause);
                clause = String::new();
            }
            Some('(') => clause = String::from('('),
            Some(c) => clause.push(c),
            None => break,
        }
    }
    clauses
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| {
            let mut chs = c.trim().chars();
            set_clause(entity_name, &mut chs)
        })
        .collect::<Vec<Clause>>()
}

fn clause_entity_definition(entity_name: &str, clause: &str) -> Clause {
    let elements = clause
        .split(' ')
        .filter(|c| !c.is_empty())
        .map(str::trim)
        .collect::<Vec<&str>>();
    if elements.len() != 3 {
        return Clause::Error;
    }

    let last_element = elements.last().unwrap();
    let entity_key = elements[1].split(':').collect::<Vec<&str>>();
    if entity_key.len() != 2 {
        return Clause::Error;
    }

    let (entity, key) = (entity_key[0], entity_key[1]);
    if entity != entity_name {
        return Clause::Error;
    }

    let mut last = last_element.chars();
    if last_element.starts_with('?') {
        Clause::ValueAttribution(
            entity.to_owned(),
            key.to_owned(),
            Value((*last_element).to_string()),
        )
    } else if let Ok(value) = parse_value(last.next().unwrap(), &mut last) {
        Clause::ContainsKeyValue(entity.to_owned(), key.to_owned(), value)
    } else {
        Clause::Error
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Clause {
    ContainsKeyValue(String, String, Types),
    ValueAttribution(String, String, Value),
    SimpleComparisonFunction(Function, String, Types),
    ComplexComparisonFunctions(Function, String, Vec<Types>),
    Or(Function, Vec<Clause>),
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Function {
    Eq,
    GEq,
    G,
    LEq,
    L,
    NotEq,
    Like,
    Between,
    Or,
    In,
    Error,
}

impl FromStr for Function {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match &s.to_lowercase()[..] {
            "==" => Self::Eq,
            ">=" => Self::GEq,
            ">" => Self::G,
            "<=" => Self::LEq,
            "<" => Self::L,
            "!=" | "<>" => Self::NotEq,
            "like" => Self::Like,
            "between" => Self::Between,
            "in" => Self::In,
            _ => Self::Error,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value(pub String);

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_error_open() {
        let mut chars = " [".chars();
        let wql = where_selector("hello".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.err(),
            Some(String::from(
                "WHERE clauses must be contained inside ` {...}`"
            ))
        );
    }

    #[test]
    fn simple_equality() {
        let mut chars = " {
            ?* my_entity:name \"julia\",
            ?* my_entity:id 349875325,
        }"
        .chars();
        let wql = where_selector("my_entity".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![
                    Clause::ContainsKeyValue(
                        "my_entity".to_string(),
                        "name".to_string(),
                        Types::String("julia".to_string())
                    ),
                    Clause::ContainsKeyValue(
                        "my_entity".to_string(),
                        "id".to_string(),
                        Types::Integer(349875325)
                    ),
                ],
                HashMap::new()
            )
        )
    }

    #[test]
    fn simple_comparison() {
        let mut chars = " {
            ?* my_entity:age ?age,
            (>= ?age 30),
            (> ?age 30),
            (== ?age 30),
            (<= ?age 30),
            (< ?age 30),
            (like ?name \"%uli%\"),
        }"
        .chars();
        let wql = where_selector("my_entity".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![
                    Clause::ValueAttribution(
                        "my_entity".to_string(),
                        "age".to_string(),
                        Value("?age".to_string())
                    ),
                    Clause::SimpleComparisonFunction(
                        Function::GEq,
                        "?age".to_string(),
                        Types::Integer(30)
                    ),
                    Clause::SimpleComparisonFunction(
                        Function::G,
                        "?age".to_string(),
                        Types::Integer(30)
                    ),
                    Clause::SimpleComparisonFunction(
                        Function::Eq,
                        "?age".to_string(),
                        Types::Integer(30)
                    ),
                    Clause::SimpleComparisonFunction(
                        Function::LEq,
                        "?age".to_string(),
                        Types::Integer(30)
                    ),
                    Clause::SimpleComparisonFunction(
                        Function::L,
                        "?age".to_string(),
                        Types::Integer(30)
                    ),
                    Clause::SimpleComparisonFunction(
                        Function::Like,
                        "?name".to_string(),
                        Types::String("%uli%".to_string())
                    ),
                ],
                HashMap::new()
            )
        )
    }

    #[test]
    fn complex_comp_func() {
        let mut chars = " {
            (in ?id 32434 45345 345346436),
            (between ?age 30 35),
        }"
        .chars();
        let wql = where_selector("my_entity".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![
                    Clause::ComplexComparisonFunctions(
                        Function::In,
                        "?id".to_string(),
                        vec![
                            Types::Integer(32434),
                            Types::Integer(45345),
                            Types::Integer(345346436),
                        ]
                    ),
                    Clause::ComplexComparisonFunctions(
                        Function::Between,
                        "?age".to_string(),
                        vec![Types::Integer(30), Types::Integer(35)]
                    )
                ],
                HashMap::new()
            )
        )
    }

    #[test]
    fn between_err() {
        let mut chars = " {
            (between ?id 32434),
            (between ?age 30 35 34),
        }"
        .chars();
        let wql = where_selector("my_entity".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![Clause::Error, Clause::Error,],
                HashMap::new()
            )
        )
    }

    #[test]
    fn or() {
        let mut chars = " {
            ?* my_entity:age ?age,
            ?* my_entity:name ?name,
            (or 
                (>= ?age 30)
                (like ?name \"%uli%\")
            ),
        }"
        .chars();
        let wql = where_selector("my_entity".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![
                    Clause::ValueAttribution(
                        "my_entity".to_string(),
                        "age".to_string(),
                        Value("?age".to_string())
                    ),
                    Clause::ValueAttribution(
                        "my_entity".to_string(),
                        "name".to_string(),
                        Value("?name".to_string())
                    ),
                    Clause::Or(
                        Function::Or,
                        vec![
                            Clause::SimpleComparisonFunction(
                                Function::GEq,
                                "?age".to_string(),
                                Types::Integer(30)
                            ),
                            Clause::SimpleComparisonFunction(
                                Function::Like,
                                "?name".to_string(),
                                Types::String("%uli%".to_string())
                            ),
                        ]
                    ),
                ],
                HashMap::new()
            )
        )
    }
}
