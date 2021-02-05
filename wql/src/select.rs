use std::str::FromStr;

use super::{logic::read_args, ToSelect, Wql};

pub(crate) fn select_all(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let arg = ToSelect::All;
    select_body(arg, chars)
}

pub(crate) fn select_args(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let args: Vec<String> = read_args(chars)?;
    let arg = ToSelect::Keys(args);

    select_body(arg, chars)
}

fn select_body(arg: ToSelect, chars: &mut std::str::Chars) -> Result<Wql, String> {
    let entity_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if entity_symbol.to_uppercase() != "FROM" {
        return Err(String::from("Keyword FROM is required for SELECT"));
    }

    let entity_name = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>();

    if entity_name.is_empty() {
        return Err(String::from("Entity name is required for SELECT"));
    }

    let id_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if id_symbol == "ID" {
        let id = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_alphanumeric() || c == &'-')
            .collect::<String>();

        let uuid = uuid::Uuid::from_str(&id);
        if uuid.is_err() {
            return Err(String::from("Field ID must be a UUID v4"));
        }
        Ok(Wql::Select(entity_name, arg, uuid.ok()))
    } else if !id_symbol.is_empty() && id_symbol != "ID" {
        Err(String::from(
            "ID keyword is required to set an uuid in SELECT",
        ))
    } else {
        Ok(Wql::Select(entity_name, arg, None))
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use crate::{ToSelect, Wql};
    use std::str::FromStr;

    #[test]
    fn select_all() {
        let wql = Wql::from_str("SelEct * FROM my_entity");

        assert_eq!(
            wql.unwrap(),
            Wql::Select("my_entity".to_string(), ToSelect::All, None)
        );
    }

    #[test]
    fn select_all_from_missing() {
        let wql = Wql::from_str("SelEct * my_entity");

        assert_eq!(
            wql.err(),
            Some(String::from("Keyword FROM is required for SELECT"))
        );
    }

    #[test]
    fn select_all_from_entity() {
        let wql = Wql::from_str("SelEct * FROM");

        assert_eq!(
            wql.err(),
            Some(String::from("Entity name is required for SELECT"))
        );
    }

    #[test]
    fn select_arg() {
        let wql = Wql::from_str("SelEct #{hello,} FROM my_entity");

        assert_eq!(
            wql.unwrap(),
            Wql::Select(
                "my_entity".to_string(),
                ToSelect::Keys(vec!["hello".to_string()]),
                None
            )
        );
    }

    #[test]
    fn select_args() {
        let wql = Wql::from_str("SelEct #{hello,world, by_me,} FROM my_entity");

        assert_eq!(
            wql.unwrap(),
            Wql::Select(
                "my_entity".to_string(),
                ToSelect::Keys(vec![
                    "hello".to_string(),
                    "world".to_string(),
                    "by_me".to_string()
                ]),
                None
            )
        );
    }

    #[test]
    fn select_all_id() {
        let wql = Wql::from_str("SelEct * FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1");
        let uuid = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1");

        assert_eq!(
            wql.unwrap(),
            Wql::Select("my_entity".to_string(), ToSelect::All, uuid.ok())
        );
    }

    #[test]
    fn select_all_id_missing() {
        let wql = Wql::from_str("SelEct * FROM my_entity ID ");

        assert_eq!(wql.err(), Some(String::from("Field ID must be a UUID v4")));
    }

    #[test]
    fn select_all_id_key_missing() {
        let wql = Wql::from_str("SelEct * FROM my_entity 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 ");

        assert_eq!(
            wql.err(),
            Some(String::from(
                "ID keyword is required to set an uuid in SELECT"
            ))
        );
    }
}
