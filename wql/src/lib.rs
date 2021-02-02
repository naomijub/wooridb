use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Wql {
    CreateEntity(String, Vec<String>),
    Insert(String, Entity),
    UpdateContent(String, Entity, Uuid),
    UpdateSet(String, Entity, Uuid),
    Delete(String, String),
}

pub type Entity = HashMap<String, Types>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Types {
    Char(char),
    Integer(isize),
    String(String),
    Uuid(Uuid),
    Float(f64),
    Boolean(bool),
    Vector(Vec<Box<Types>>),
    Map(HashMap<String, Box<Types>>),
    //DateTime
    Nil,
}

pub(crate) fn tokenize(wql: &str) -> std::str::Chars {
    wql.chars()
}

impl std::str::FromStr for Wql {
    type Err = String;

    /// Parses a `&str` that contains an Edn into `Result<Edn, EdnError>`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = tokenize(s.trim_start());
        let wql = parse(tokens.next(), &mut tokens)?;
        Ok(wql)
    }
}

pub(crate) fn parse(c: Option<char>, chars: &mut std::str::Chars) -> Result<Wql, String> {
    if c.is_some() {
        read_symbol(c.unwrap(), chars)
    } else {
        Err(String::from("Empty WQL"))
    }
}

fn read_symbol(a: char, chars: &mut std::str::Chars) -> Result<Wql, String> {
    let symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();

    match (a, &symbol.to_uppercase()[..]) {
        ('c', "REATE") | ('C', "REATE") => create_entity(chars),
        ('i', "NSERT") | ('I', "NSERT") => insert(chars),
        ('u', "PDATE") | ('U', "PDATE") => update(chars),
        ('d', "ELETE") | ('D', "ELETE") => delete(chars),
        _ => Err(format!("Symbol `{}{}` not implemented", a, symbol)),
    }
}

fn create_entity(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let entity_symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();

    if entity_symbol.to_uppercase() != String::from("ENTITY") {
        return Err(String::from("Keyword ENTITY is required for CREATE"));
    }

    let entity_name = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    let unique_symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();
    if unique_symbol.to_uppercase() == String::from("UNIQUES") {
        let unique_names = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| {
                c.is_alphanumeric() || c == &'_' || c == &',' || c.is_whitespace() || c != &';'
            })
            .collect::<String>()
            .trim()
            .to_string();

        let unique_vec = unique_names
            .split(",")
            .map(|w| w.trim().to_string())
            .collect::<Vec<String>>();

        Ok(Wql::CreateEntity(entity_name, unique_vec))
    } else {
        Ok(Wql::CreateEntity(entity_name, Vec::new()))
    }
}

fn delete(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let entity_id = chars
        .take_while(|c| c.is_alphanumeric() || c == &'-')
        .collect::<String>()
        .trim()
        .to_string();

    if entity_id.is_empty() || entity_id == "FROM" {
        return Err(String::from("Entity UUID is required for DELETE"));
    }

    let entity_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if entity_symbol.to_uppercase() != String::from("FROM") {
        return Err(String::from("Keyword FROM is required for DELETE"));
    }

    let entity_name = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    if entity_name.is_empty() {
        return Err(String::from("Entity name is required after FROM"));
    }

    Ok(Wql::Delete(entity_name, entity_id))
}

fn insert(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let entity_map = read_map(chars)?;
    let entity_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if entity_symbol.to_uppercase() != String::from("INTO") {
        return Err(String::from("Keyword INTO is required for INSERT"));
    }

    let entity_name = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    if entity_name.is_empty() {
        return Err(String::from("Entity name is required after INTO"));
    }

    Ok(Wql::Insert(entity_name, entity_map))
}

fn update(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let entity_name = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    if entity_name.is_empty() {
        return Err(String::from("Entity name is required for UPDATE"));
    };

    let entity_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if entity_symbol.to_uppercase() != "SET" && entity_symbol.to_uppercase() != "CONTENT" {
        return Err(String::from(
            "UPDATE type is required after entity. Keywords are SET or CONTENT",
        ));
    };

    let entity_map = read_map(chars)?;

    let into_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if into_symbol.to_uppercase() != String::from("INTO") {
        return Err(String::from("Keyword INTO is required for UPDATE"));
    };

    let uuid_str = chars
        .take_while(|c| c.is_alphanumeric() || c == &'-')
        .collect::<String>()
        .trim()
        .to_string();

    let uuid =
        Uuid::from_str(&uuid_str).map_err(|_| format!("Couldn't create uuid from {}", uuid_str))?;

    match &entity_symbol.to_uppercase()[..] {
        "SET" => Ok(Wql::UpdateSet(entity_name, entity_map, uuid)),
        "CONTENT" => Ok(Wql::UpdateContent(entity_name, entity_map, uuid)),
        _ => Err("Couldn't parse UPDATE query".to_string()),
    }
}

fn read_map(chars: &mut std::str::Chars) -> Result<HashMap<String, Types>, String> {
    let mut res: HashMap<String, Types> = HashMap::new();
    let mut key: Option<String> = None;
    let mut val: Option<Types> = None;

    if chars.next() != Some('{') {
        return Err(String::from(
            "Entity map should start with `{` and end with `}`",
        ));
    }

    loop {
        match chars.next() {
            Some('}') => return Ok(res),
            Some(c) if !c.is_whitespace() && c != ',' => {
                if key.is_some() {
                    val = Some(parse_value(c, chars)?);
                } else {
                    key = Some(parse_key(c, chars));
                }
            }
            Some(c) if c.is_whitespace() || c == ',' => (),
            _ => return Err(String::from("Entity HashMap could not be created")),
        }

        if key.is_some() && val.is_some() {
            res.insert(key.unwrap().to_string(), val.unwrap());
            key = None;
            val = None;
        }
    }
}

fn parse_key(c: char, chars: &mut std::str::Chars) -> String {
    let key_rest = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>();
    format!("{}{}", c, key_rest)
}

pub(crate) fn parse_value(c: char, chars: &mut std::str::Chars) -> Result<Types, String> {
    if c == '"' {
        return read_str(chars);
    }

    let value = format!(
        "{}{}",
        c,
        chars
            .take_while(|c| !c.is_whitespace() && c != &',')
            .collect::<String>()
    );

    if value.parse::<isize>().is_ok() {
        Ok(Types::Integer(value.parse().unwrap()))
    } else if value.parse::<f64>().is_ok() {
        Ok(Types::Float(value.parse().unwrap()))
    } else if uuid::Uuid::from_str(&value).is_ok() {
        Ok(Types::Uuid(uuid::Uuid::from_str(&value).unwrap()))
    } else if value.parse::<bool>().is_ok() {
        Ok(Types::Boolean(value.parse().unwrap()))
    } else if &value.to_lowercase() == "nil" {
        Ok(Types::Nil)
    } else if value.starts_with("'") && value.ends_with("'") && value.len() == 3 {
        Ok(Types::Char(value.chars().nth(1).unwrap()))
    } else {
        Err(format!("Value Type could not be created from {}", value))
    }
}

fn read_str(chars: &mut std::str::Chars) -> Result<Types, String> {
    let result = chars.try_fold((false, String::new()), |(last_was_escape, mut s), c| {
        if last_was_escape {
            // Supported escape characters, per https://github.com/edn-format/edn#strings
            match c {
                't' => s.push('\t'),
                'r' => s.push('\r'),
                'n' => s.push('\n'),
                '\\' => s.push('\\'),
                '\"' => s.push('\"'),
                _ => return Err(Err(format!("Invalid escape sequence \\{}", c))),
            };

            Ok((false, s))
        } else if c == '\"' {
            // Unescaped quote means we're done
            Err(Ok(s))
        } else if c == '\\' {
            Ok((true, s))
        } else {
            s.push(c);
            Ok((false, s))
        }
    });

    match result {
        // An Ok means we actually finished parsing *without* seeing the end of the string, so that's
        // an error.
        Ok(_) => Err("Unterminated string".to_string()),
        Err(Err(e)) => Err(e),
        Err(Ok(string)) => Ok(Types::String(string)),
    }
}

#[cfg(test)]
mod test_create {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn empty_wql() {
        let wql = Wql::from_str("");

        assert_eq!(wql.err(), Some(String::from("Empty WQL")));
    }

    #[test]
    fn create_shit() {
        let wql = Wql::from_str("CREATE SHIT oh_yeah");

        assert_eq!(
            wql.err(),
            Some(String::from("Keyword ENTITY is required for CREATE"))
        );
    }

    #[test]
    fn create_mispelled() {
        let wql = Wql::from_str("KREATE ENTITY mispelled");

        assert_eq!(
            wql.err(),
            Some(String::from("Symbol `KREATE` not implemented"))
        );
    }

    #[test]
    fn create_entity() {
        let wql = Wql::from_str("CREATE ENTITY entity");

        assert_eq!(
            wql.unwrap(),
            Wql::CreateEntity(String::from("entity"), Vec::new())
        );
    }

    #[test]
    fn create_entity_with_uniques() {
        let wql = Wql::from_str("CREATE ENTITY entity UNIQUES name, ssn, something");

        assert_eq!(
            wql.unwrap(),
            Wql::CreateEntity(
                String::from("entity"),
                vec![
                    "name".to_string(),
                    "ssn".to_string(),
                    "something".to_string()
                ]
            )
        );
    }
}

#[cfg(test)]
mod test_delete {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn delete_id() {
        let wql = Wql::from_str("DELETE this-is-an-uuid FROM my_entity");

        assert_eq!(
            wql.unwrap(),
            Wql::Delete("my_entity".to_string(), "this-is-an-uuid".to_string())
        )
    }

    #[test]
    fn delete_missing_id() {
        let wql = Wql::from_str("DELETE FROM my_entity");

        assert_eq!(
            wql.err(),
            Some(String::from("Entity UUID is required for DELETE"))
        );
    }

    #[test]
    fn delete_missing_keyword_from() {
        let wql = Wql::from_str("DELETE this-is-an-uuid my_entity");

        assert_eq!(
            wql.err(),
            Some(String::from("Keyword FROM is required for DELETE"))
        );
    }

    #[test]
    fn delete_missing_entity() {
        let wql = Wql::from_str("DELETE this-is-an-uuid FROM");

        assert_eq!(
            wql.err(),
            Some(String::from("Entity name is required after FROM"))
        );
    }
}

#[cfg(test)]
mod test_insert {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn insert_entity() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
            b: 12.3,
            c: 'd' ,
            d: true ,
            e: false,
            f: \"hello\",
            g: NiL
        } INTO my_entity",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::Insert("my_entity".to_string(), hashmap())
        );
    }

    #[test]
    fn insert_missing_into() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
        } INTRO my_entity",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Keyword INTO is required for INSERT"))
        );
    }

    #[test]
    fn insert_missing_entity_name() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
        } INTO ",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Entity name is required after INTO"))
        );
    }

    fn hashmap() -> Entity {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Integer(123));
        hm.insert("b".to_string(), Types::Float(12.3));
        hm.insert("c".to_string(), Types::Char('d'));
        hm.insert("d".to_string(), Types::Boolean(true));
        hm.insert("e".to_string(), Types::Boolean(false));
        hm.insert("f".to_string(), Types::String("hello".to_string()));
        hm.insert("g".to_string(), Types::Nil);
        hm
    }
}

#[cfg(test)]
mod test_update {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn update_set_entity() {
        let wql = Wql::from_str(
            "UPDATE this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::UpdateSet(
                "this_entity".to_string(),
                hashmap(),
                Uuid::from_str("d6ca73c0-41ff-4975-8a60-fc4a061ce536").unwrap()
            )
        );
    }

    #[test]
    fn update_content_entity() {
        let wql = Wql::from_str(
            "UPDATE this_entity 
        Content {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::UpdateContent(
                "this_entity".to_string(),
                hashmap(),
                Uuid::from_str("d6ca73c0-41ff-4975-8a60-fc4a061ce536").unwrap()
            )
        );
    }

    #[test]
    fn update_set_missing_entity() {
        let wql = Wql::from_str(
            "UPDATE 
        SET {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Entity name is required for UPDATE"))
        );
    }

    fn hashmap() -> Entity {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Integer(123));
        hm.insert("g".to_string(), Types::Nil);
        hm
    }

    #[test]
    fn update_entity_mispelled_action() {
        let wql = Wql::from_str(
            "UPDATE this_entity 
        TO {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err(),
            Some(String::from(
                "UPDATE type is required after entity. Keywords are SET or CONTENT"
            ))
        );
    }

    #[test]
    fn update_entity_missing_into() {
        let wql = Wql::from_str(
            "UPDATE this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        to d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Keyword INTO is required for UPDATE"))
        );
    }

    #[test]
    fn update_entity_missing_uuid() {
        let wql = Wql::from_str(
            "UPDATE this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        into Some-crazy-id",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Couldn\'t create uuid from Some-crazy-id"))
        );
    }
}
