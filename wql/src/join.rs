use std::str::FromStr;

use crate::Wql;

pub fn join(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let mut entity_a = (String::new(), String::new());
    let mut entity_b = (String::new(), String::new());

    let mut ent = String::new();
    let mut key = String::new();
    let mut is_entity = true;
    loop {
        match chars.next() {
            Some(' ') | Some('(') => (),
            Some(c) if c.is_alphanumeric() || c == '_' => {
                if is_entity {
                    ent.push(c);
                } else {
                    key.push(c);
                }
            }
            Some(':') => {
                is_entity = false;
                if entity_a.0.is_empty() {
                    entity_a.0 = ent;
                    ent = String::new();
                } else {
                    entity_b.0 = ent;
                    ent = String::new();
                }
            }
            Some(',') => {
                is_entity = true;
                if entity_a.1.is_empty() {
                    entity_a.1 = key;
                    key = String::new();
                } else {
                    entity_b.1 = key;
                    key = String::new();
                }
            }
            Some(')') => {
                entity_b.1 = key;
                break;
            }
            _ => return Err(String::from("Invalid char for Join")),
        }
    }

    let queries = chars
        .skip_while(|c| c == &'(' || c.is_whitespace())
        .take_while(|c| c != &')')
        .collect::<String>();

    let queries = queries.split('|').collect::<Vec<&str>>();

    if queries.len() != 2 {
        return Err(String::from("Join can only support 2 select queries"));
    } else if !queries[0].contains(&entity_a.0) {
        return Err(format!(
            "{} must be present as entity tree key in `SELECT * FROM {}`",
            entity_a.0, queries[0]
        ));
    } else if !queries[1].contains(&entity_b.0) {
        return Err(format!(
            "{} must be present as entity tree key in `SELECT * FROM {}`",
            entity_b.0, queries[1]
        ));
    }

    let queries_wql = queries
        .into_iter()
        .map(|q| Wql::from_str(q))
        .collect::<Result<Vec<Wql>, String>>()?;

    // WITH clause

    Ok(Wql::Join(entity_a, entity_b, queries_wql))
}

#[cfg(test)]
mod test {

    use crate::{ToSelect, Wql};
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn test_join() {
        let wql = Wql::from_str(
            "JOIN (entity_A:c, entity_B:c) Select * FROM entity_A | Select * FROM entity_B",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::Join(
                ("entity_A".to_string(), "c".to_string()),
                ("entity_B".to_string(), "c".to_string()),
                vec![
                    Wql::Select("entity_A".to_string(), ToSelect::All, None, HashMap::new()),
                    Wql::Select("entity_B".to_string(), ToSelect::All, None, HashMap::new())
                ]
            )
        )
    }
}
