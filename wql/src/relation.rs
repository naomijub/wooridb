use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::Wql;
const ERROR: &str = "Supported operations for INTERSECT and DIFFERECE are KEY for mathching keys and KEY_VALUE for matching key_values";

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Relation {
    Difference,
    Intersect,
    Union,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RelationType {
    Key,
    KeyValue,
}

impl FromStr for RelationType {
    fn from_str(s: &str) -> Result<Self, String> {
        match &s.to_uppercase()[..] {
            "KEY" => Ok(Self::Key),
            "KEY-VALUE" => Ok(Self::KeyValue),
            _ => Err(String::from(ERROR)),
        }
    }

    type Err = String;
}

const POSSIBLE_RELATION_TYPES: [&str; 2] = ["KEY", "KEY-VALUE"];

pub fn relation(chars: &mut std::str::Chars, relation: Relation) -> Result<Wql, String> {
    let type_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();

    if !POSSIBLE_RELATION_TYPES.contains(&&type_symbol[..]) {
        return Err(String::from(ERROR));
    }

    let all: String = chars.collect();
    let queries: Vec<&str> = all.split('|').collect();
    if queries.len() != 2 {
        return Err(String::from(
            "Intersect and difference should have exactly 2 queries",
        ));
    }

    let queries = queries
        .into_iter()
        .map(|q| Wql::from_str(q))
        .collect::<Result<Vec<Wql>, String>>()?;

    let queries = queries.into_iter()
        .map(|q| {
            match &q {
                Wql::Select(_, _, Some(_), hm) if hm.is_empty()
                    => Ok(q),
                Wql::SelectWhen(_, _, Some(_), _)
                    => Ok(q),
                _ => Err(String::from("Only single value queries are allowed, so key `ID` is required and keys `WHEN AT` are optional"))
            }
        })
        .collect::<Result<Vec<Wql>, String>>()?;
    let operation = RelationType::from_str(&type_symbol)?;
    Ok(Wql::RelationQuery(queries, relation, operation))
}
