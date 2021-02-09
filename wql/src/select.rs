use std::str::FromStr;

use uuid::Uuid;

use super::{
    logic::{read_select_args, read_uuids},
    ToSelect, Wql,
};

pub(crate) fn select_all(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let arg = ToSelect::All;
    select_body(arg, chars)
}

pub(crate) fn select_args(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let args: Vec<String> = read_select_args(chars)?;
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

    let next_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();

    if next_symbol == "ID" {
        let id = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_alphanumeric() || c == &'-')
            .collect::<String>();

        let uuid = uuid::Uuid::from_str(&id);
        if uuid.is_err() {
            return Err(String::from("Field ID must be a UUID v4"));
        }
        let next_symbol = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase();
        if next_symbol.to_uppercase() == "WHEN" {
            return when_selector(entity_name, arg, uuid.ok(), chars);
        }

        Ok(Wql::Select(entity_name, arg, uuid.ok()))
    } else if next_symbol == "IDS" {
        let in_symbol = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase();

        if in_symbol == "IN" {
            let uuids: Vec<Uuid> = read_uuids(chars)?;
            let next_symbol = chars
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| !c.is_whitespace())
                .collect::<String>()
                .to_uppercase();
            if next_symbol.to_uppercase() == "WHEN" {
                return Err(String::from("WHEN not allowed after IDS IN"));
            }
            Ok(Wql::SelectIds(entity_name, arg, uuids))
        } else {
            Err(String::from(
                "IN keyword is required after IDS to define a set of uuids",
            ))
        }
    } else if next_symbol.to_uppercase() == "WHEN" {
        when_selector(entity_name, arg, None, chars)
    } else if !next_symbol.is_empty()
        && (next_symbol.to_uppercase() != "ID" || next_symbol.to_uppercase() != "IDS")
    {
        println!("{:?}", next_symbol);
        Err(String::from(
            "ID/IDS keyword is required to set an uuid in SELECT",
        ))
    } else {
        Ok(Wql::Select(entity_name, arg, None))
    }
}

fn when_selector(
    entity_name: String,
    arg: ToSelect,
    uuid: Option<Uuid>,
    chars: &mut std::str::Chars,
) -> Result<Wql, String> {
    let next_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();

    if let (&ToSelect::All, Some(uuid), "START") = (&arg, uuid, next_symbol.to_uppercase().as_str())
    {
        return when_time_range(entity_name, uuid, chars);
    }
    if next_symbol.to_uppercase() != "AT" {
        return Err(String::from("AT is required after WHEN"));
    };

    let date = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    Ok(Wql::SelectWhen(entity_name, arg, uuid, date))
}

fn when_time_range(
    entity_name: String,
    uuid: Uuid,
    chars: &mut std::str::Chars,
) -> Result<Wql, String> {
    let start_date = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    let next_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();
    if next_symbol.to_uppercase() != "END" {
        return Err(String::from(
            "END is required after START date for SELECT WHEN",
        ));
    };

    let end_date = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if !end_date.starts_with(&start_date[0..10]) {
        return Err(String::from(
            "START date and END date should be the same date.",
        ));
    }

    Ok(Wql::SelectWhenRange(
        entity_name,
        uuid,
        start_date,
        end_date,
    ))
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

    #[test]
    fn when_at() {
        let wql = Wql::from_str("SelEct * FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 WHEN AT 2020-01-01T00:00:00Z");
        let uuid = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").unwrap();
        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhen(
                "my_entity".to_string(),
                ToSelect::All,
                Some(uuid),
                "2020-01-01T00:00:00Z".to_string()
            )
        );
    }

    #[test]
    fn when_at_args() {
        let wql = Wql::from_str("SelEct #{a,b,c,} FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 WHEN AT 2020-01-01T00:00:00Z");
        let uuid = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").unwrap();
        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhen(
                "my_entity".to_string(),
                ToSelect::Keys(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
                Some(uuid),
                "2020-01-01T00:00:00Z".to_string()
            )
        );
    }

    #[test]
    fn when_at_args_no_id() {
        let wql = Wql::from_str("SelEct #{a,b,c,} FROM my_entity WHEN AT 2020-01-01T00:00:00Z");

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhen(
                "my_entity".to_string(),
                ToSelect::Keys(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
                None,
                "2020-01-01T00:00:00Z".to_string()
            )
        );
    }

    #[test]
    fn when_range_all() {
        let wql = Wql::from_str("SelEct * FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 WHEN START 2020-01-01T00:00:00Z END 2020-01-01T03:00:00Z");
        let uuid = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").unwrap();
        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhenRange(
                "my_entity".to_string(),
                uuid,
                "2020-01-01T00:00:00Z".to_string(),
                "2020-01-01T03:00:00Z".to_string()
            )
        );
    }

    #[test]
    fn when_range_args_err() {
        let wql = Wql::from_str("SelEct * FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 WHEN START 2020-01-01T00:00:00Z 2020-01-01T03:00:00Z");

        assert_eq!(
            wql.err(),
            Some(String::from(
                "END is required after START date for SELECT WHEN"
            ))
        );
    }
}
