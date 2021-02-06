use std::str::FromStr;

use uuid::Uuid;

use super::{
    logic::{read_args, read_uuids},
    ToSelect, Wql,
};

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
        .collect::<String>()
        .to_uppercase();

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
    } else if id_symbol == "IDS" {
        let in_symbol = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase();

        if in_symbol != "IN" {
            Err(String::from(
                "IN keyword is required after IDS to define a set of uuids",
            ))
        } else {
            let uuids: Vec<Uuid> = read_uuids(chars)?;
            Ok(Wql::SelectIds(entity_name, arg, uuids))
        }
    } else if !id_symbol.is_empty() && (id_symbol != "ID" || id_symbol != "IDS") {
        Err(String::from(
            "ID/IDS keyword is required to set an uuid in SELECT",
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
                "ID/IDS keyword is required to set an uuid in SELECT"
            ))
        );
    }

    #[test]
    fn select_all_ids() {
        let wql = Wql::from_str("SelEct * FROM my_entity IDS IN #{2df2b8cf-49da-474d-8a00-c596c0bb6fd1, 53315090-e14d-4738-a4d2-f1ec2a93664c,}");
        let uuid1 = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").unwrap();
        let uuid2 = Uuid::from_str("53315090-e14d-4738-a4d2-f1ec2a93664c").unwrap();

        assert_eq!(
            wql.unwrap(),
            Wql::SelectIds("my_entity".to_string(), ToSelect::All, vec![uuid1, uuid2])
        );
    }

    #[test]
    fn select_keys_ids() {
        let wql = Wql::from_str("SelEct #{a, b, c,} FROM my_entity IDS IN #{2df2b8cf-49da-474d-8a00-c596c0bb6fd1, 53315090-e14d-4738-a4d2-f1ec2a93664c,}");
        let uuid1 = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").unwrap();
        let uuid2 = Uuid::from_str("53315090-e14d-4738-a4d2-f1ec2a93664c").unwrap();

        assert_eq!(
            wql.unwrap(),
            Wql::SelectIds(
                "my_entity".to_string(),
                ToSelect::Keys(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
                vec![uuid1, uuid2]
            )
        );
    }

    #[test]
    fn select_all_ids_missing_in() {
        let wql = Wql::from_str("SelEct * FROM my_entity IDS #{2df2b8cf-49da-474d-8a00-c596c0bb6fd1, 53315090-e14d-4738-a4d2-f1ec2a93664c,}");

        assert_eq!(
            wql.err(),
            Some(String::from(
                "IN keyword is required after IDS to define a set of uuids"
            ))
        );
    }
}
