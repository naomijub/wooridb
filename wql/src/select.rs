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

    Ok(Wql::Select(entity_name, arg))
}

#[cfg(test)]
mod test {
    use crate::{ToSelect, Wql};
    use std::str::FromStr;

    #[test]
    fn select_all() {
        let wql = Wql::from_str("SelEct * FROM my_entity");

        assert_eq!(
            wql.unwrap(),
            Wql::Select("my_entity".to_string(), ToSelect::All)
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
                ToSelect::Keys(vec!["hello".to_string()])
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
                ])
            )
        );
    }
}
