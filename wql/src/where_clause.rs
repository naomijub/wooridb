use crate::{ToSelect, Types, Wql, logic::parse_value};
use serde::{Serialize, Deserialize};

pub fn where_selector(entity_name: String, arg: ToSelect, chars: &mut std::str::Chars) -> Result<Wql, String> {
    let mut open = chars
        .skip_while(|c| c.is_whitespace())
        .take(1);
    
    if open.next() != Some('{') {
        return Err(String::from("WHERE clauses must be contained inside ` {...}`"));
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

    let clauses = clauses.into_iter()
        .filter(|c| !c.is_empty())
        .map(|c| {
            let mut chs = c.trim().chars();
            set_clause(&entity_name, &mut chs)
        })
        .collect::<Vec<Clause>>();
    if clauses.is_empty() {
        return Err(String::from("WHERE clause cannot be empty"))
    }

    Ok(Wql::SelectWhere(entity_name, arg, clauses))
}

fn set_clause(entity_name: &str, chs: &mut std::str::Chars) -> Clause {
    let c_str: String = chs.skip_while(|c| c.is_whitespace()).take_while(|c| c != &',').collect();
    if c_str.starts_with("?*") {
        let elements = c_str.split(' ').filter(|c| !c.is_empty()).map(|c| c.trim()).collect::<Vec<&str>>();
        if elements.len() != 3 { return Clause::Error; }

        let entity_key = elements[1].split(':').collect::<Vec<&str>>();
        if entity_key.len() != 2 { return Clause::Error; }

        let (entity, key) = (entity_key[0], entity_key[1]);
        if entity != entity_name { return Clause::Error; }
        
        let last_element = elements.last().unwrap();
        let mut last = last_element.chars();
        if last_element.starts_with('?') {
            Clause::ValueAttribution(entity.to_owned(), key.to_owned(), Value(last_element.to_string()))
        } else if let Ok(value) = parse_value(last.next().unwrap(), &mut last) {
            Clause::ContainsKeyValue(entity.to_owned(), key.to_owned(), value)
        } else {
            Clause::Error
        }
    } else {
        Clause::Error
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Clause {
    ContainsKeyValue(String, String, Types),
    ValueAttribution(String, String, Value),
    Error
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
            Some(String::from("WHERE clauses must be contained inside ` {...}`"))
        );
    }

    #[test]
    fn simple_equality() {
        let mut chars = " {
            ?* my_entity:name \"julia\",
            ?* my_entity:id 349875325,
        }".chars();
        let wql = where_selector("my_entity".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere("my_entity".to_string(), ToSelect::All,
                vec![
                    Clause::ContainsKeyValue("my_entity".to_string(),  "name".to_string(), Types::String("julia".to_string())),
                    Clause::ContainsKeyValue("my_entity".to_string(),  "id".to_string(), Types::Integer(349875325)),
                ]
            )
        )
    }

    #[test]
    fn var_attribution() {
        let mut chars = " {
            ?* my_entity:id ?id,
        }".chars();
        let wql = where_selector("my_entity".to_string(), ToSelect::All, &mut chars);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere("my_entity".to_string(), ToSelect::All,
                vec![
                    Clause::ValueAttribution("my_entity".to_string(),  "id".to_string(), Value("?id".to_string())),
                ]
            )
        )
    }

    // #[test]
    // fn debug() {
    //     let mut chars = " {
    //         ?* my_entity:name \"julia\",
    //         ?* my_entity:id 349875325,
    //         ?* my_entity:id ?id,
    //         (in ?id [32434, 45345, 345346436]),
    //         ?* my_entity:age ?age,
    //         (>= ?age 30),
    //         (> ?age 30),
    //         (== ?age 30),
    //         (<= ?age 30),
    //         (< ?age 30),
    //         (between ?age 30 35),
    //         ?* my_entity:name ?name,
    //         (like ?name \"%uli%\"),
    //         ?* other_entity:id ?id,
    //     }".chars();
    //     let wql = where_selector("hello".to_string(), ToSelect::All, &mut chars);

    //     assert_eq!(
    //         wql.err(),
    //         Some(String::from("Where clauses must be contained inside ` {...}`"))
    //     )
    // }
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