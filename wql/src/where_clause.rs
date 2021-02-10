use std::str::FromStr;

use crate::{logic::parse_value, ToSelect, Types, Wql};
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
            Some('}') => break,
            Some(c) => clause.push(c),
            None => break,
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

    Ok(Wql::SelectWhere(entity_name, arg, clauses))
}

fn set_clause(entity_name: &str, chs: &mut std::str::Chars) -> Clause {
    let c_str: String = chs
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c != &',')
        .collect();
    if c_str.starts_with("?*") {
        clause_entity_definition(entity_name, c_str)
    } else if c_str.starts_with('(') && c_str.ends_with(')') {
        clause_function(&c_str[1..c_str.len() - 1])
    } else {
        Clause::Error
    }
}
fn clause_function(clause: &str) -> Clause {
    let args: Vec<&str> = clause
        .split(' ')
        .filter(|c| !c.is_empty())
        .map(|c| c.trim())
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
        "in" | "between" => Clause::Error,
        _ => Clause::Error,
    }
}

fn clause_entity_definition(entity_name: &str, clause: String) -> Clause {
    let elements = clause
        .split(' ')
        .filter(|c| !c.is_empty())
        .map(|c| c.trim())
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
        return Clause::EntitiesKeyComparison(
            entity_name.to_string(),
            entity.to_string(),
            key.to_string(),
            Value(last_element.to_string()),
        );
    }

    let mut last = last_element.chars();
    if last_element.starts_with('?') {
        Clause::ValueAttribution(
            entity.to_owned(),
            key.to_owned(),
            Value(last_element.to_string()),
        )
    } else if let Ok(value) = parse_value(last.next().unwrap(), &mut last) {
        Clause::ContainsKeyValue(entity.to_owned(), key.to_owned(), value)
    } else {
        Clause::Error
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Clause {
    ContainsKeyValue(String, String, Types),
    ValueAttribution(String, String, Value),
    EntitiesKeyComparison(String, String, String, Value),
    SimpleComparisonFunction(Function, String, Types),
    Error,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Function {
    Eq,
    GEq,
    G,
    LEq,
    L,
    NotEq,
    Like,
    Betweem,
    In,
    Error,
}

impl FromStr for Function {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match &s.to_lowercase()[..] {
            "==" => Function::Eq,
            ">=" => Function::GEq,
            ">" => Function::G,
            "<=" => Function::LEq,
            "<" => Function::L,
            "!=" => Function::NotEq,
            "<>" => Function::NotEq,
            "like" => Function::Like,
            "between" => Function::Betweem,
            "in" => Function::In,
            _ => Function::Error,
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Value(pub String);

#[cfg(test)]
mod test {
    use super::*;

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
                ]
            )
        )
    }

    #[test]
    fn var_attribution() {
        let mut chars = " {
            ?* my_entity:id ?id,
            ?* other_entity:id ?id,
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
                        "id".to_string(),
                        Value("?id".to_string())
                    ),
                    Clause::EntitiesKeyComparison(
                        "my_entity".to_string(),
                        "other_entity".to_string(),
                        "id".to_string(),
                        Value("?id".to_string())
                    )
                ]
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
                ]
            )
        )
    }
}

// SELECT *
// FROM my_entity
// WHERE {
//     (in ?id [32434, 45345, 345346436]),
//     ?* my_entity:age ?age,
//     (>= ?age 30),
//     (> ?age 30),
//     (== ?age 30),
//     (<= ?age 30),
//     (< ?age 30),
//     (between ?age 30 35),
//     ?* my_entity:name ?name,
//     (like ?name "%uli%"),
//     ?* other_entity:id ?id,
// }
