use crate::{
    logic::{read_args, read_map_as_str},
    select::{select_all, select_args},
};

use super::{read_map, read_match_args, FromStr, MatchCondition, Uuid, Wql};

pub(crate) fn read_symbol(a: char, chars: &mut std::str::Chars) -> Result<Wql, String> {
    let symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();

    match (a, &symbol.to_uppercase()[..]) {
        ('c', "REATE") | ('C', "REATE") => create_entity(chars),
        ('i', "NSERT") | ('I', "NSERT") => insert(chars),
        ('u', "PDATE") | ('U', "PDATE") => update(chars),
        ('d', "ELETE") | ('D', "ELETE") => delete(chars),
        ('m', "ATCH") | ('M', "ATCH") => match_update(chars),
        ('e', "VICT") | ('E', "VICT") => evict(chars),
        ('s', "ELECT") | ('S', "ELECT") => select(chars),
        ('c', "HECK") | ('C', "HECK") => check(chars),
        _ => Err(format!("Symbol `{}{}` not implemented", a, symbol)),
    }
}

fn create_entity(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let entity_symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();

    if entity_symbol.to_uppercase() != "ENTITY" {
        return Err(String::from("Keyword ENTITY is required for CREATE"));
    }

    let entity_name = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    let next_symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();
    if next_symbol.to_uppercase() == "UNIQUES" {
        let (uniques, encrypts) = create_uniques_and_encrypts(chars, "ENCRYPT")?;

        Ok(Wql::CreateEntity(entity_name, uniques, encrypts))
    } else if next_symbol.to_uppercase() == "ENCRYPT" {
        let (encrypts, uniques) = create_uniques_and_encrypts(chars, "UNIQUES")?;

        Ok(Wql::CreateEntity(entity_name, uniques, encrypts))
    } else if next_symbol.to_uppercase() == "ENCRYPTS" {
        Err(String::from("Correct wording is ENCRYPT"))
    } else if next_symbol.to_uppercase() == "UNIQUE" {
        Err(String::from("Correct wording is UNIQUES"))
    } else {
        Ok(Wql::CreateEntity(entity_name, Vec::new(), Vec::new()))
    }
}

fn create_uniques_and_encrypts(
    chars: &mut std::str::Chars,
    next_element: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut aux_vec = Vec::new();
    if chars.next() != Some('#') {
        return Err(String::from(
            "Arguments set should start with `#{` and end with `}`",
        ));
    }
    let main_vec = read_args(chars)?;
    let encrypt_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();
    if encrypt_symbol.to_uppercase() == next_element {
        if chars.next() != Some('#') {
            return Err(String::from(
                "Arguments set should start with `#{` and end with `}`",
            ));
        }
        aux_vec = read_args(chars)?;
    }

    if aux_vec.iter().any(|e| main_vec.contains(e)) {
        return Err(String::from("Encrypted arguments cannot be set to UNIQUE"));
    }
    Ok((main_vec, aux_vec))
}

fn select(chars: &mut std::str::Chars) -> Result<Wql, String> {
    loop {
        match chars.next() {
            Some(' ') => (),
            Some('*') => return select_all(chars),
            Some('#') => return select_args(chars),
            _ => return Err(String::from("SELECT expression should be followed by `*` for ALL keys or `#{key_names...}` for some keys"))
        }
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

    if entity_symbol.to_uppercase() != "FROM" {
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

    if entity_symbol.to_uppercase() != "INTO" {
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

    Ok(Wql::Insert(entity_name, entity_map, None))
}

fn check(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let entity_map = read_map_as_str(chars)?;
    let entity_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if entity_symbol.to_uppercase() != "FROM" {
        return Err(String::from("Keyword FROM is required for CHECK"));
    }

    let entity_name = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    if entity_name.is_empty() {
        return Err(String::from("Entity name is required after FROM"));
    }

    let id_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if id_symbol.to_uppercase() != "ID" {
        return Err(String::from("Keyword FROM is required for CHECK"));
    }
    let entity_id = chars
        .take_while(|c| c.is_alphanumeric() || c == &'-')
        .collect::<String>()
        .trim()
        .to_owned();
    let id = Uuid::from_str(&entity_id).map_err(|e| format!("{:?}", e))?;

    Ok(Wql::CheckValue(entity_name, id, entity_map))
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

    if into_symbol.to_uppercase() != "INTO" {
        return Err(String::from("Keyword INTO is required for UPDATE"));
    };

    let uuid_str = chars
        .take_while(|c| c.is_alphanumeric() || c == &'-')
        .collect::<String>()
        .trim()
        .to_string();

    let uuid = Uuid::from_str(&uuid_str)
        .map_err(|e| format!("Couldn't create uuid from {}. Error: {:?}", uuid_str, e))?;

    match &entity_symbol.to_uppercase()[..] {
        "SET" => Ok(Wql::UpdateSet(entity_name, entity_map, uuid)),
        "CONTENT" => Ok(Wql::UpdateContent(entity_name, entity_map, uuid)),
        _ => Err("Couldn't parse UPDATE query".to_string()),
    }
}

fn match_update(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let match_arg_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();

    if &match_arg_symbol.to_uppercase() != "ALL" && &match_arg_symbol.to_uppercase() != "ANY" {
        return Err(String::from("MATCH requires ALL or ANY symbols"));
    }

    let logical_args = read_match_args(chars)?;

    let match_args = if match_arg_symbol.to_uppercase().eq("ALL") {
        Ok(MatchCondition::All(logical_args))
    } else if match_arg_symbol.to_uppercase().eq("ANY") {
        Ok(MatchCondition::Any(logical_args))
    } else {
        Err(String::from("MATCH requires ALL or ANY symbols"))
    };

    let update_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();

    if update_symbol.to_uppercase() != "UPDATE" {
        return Err(String::from("UPDATE keyword is required for MATCH UPDATE"));
    };

    let entity_name = chars
        .take_while(|c| c.is_alphanumeric() || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    if entity_name.is_empty() {
        return Err(String::from("Entity name is required for MATCH UPDATE"));
    };

    let entity_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if entity_symbol.to_uppercase() != "SET" {
        return Err(String::from(
            "MATCH UPDATE type is required after entity. Keyword is SET",
        ));
    };

    let entity_map = read_map(chars)?;

    let into_symbol = chars
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();

    if into_symbol.to_uppercase() != "INTO" {
        return Err(String::from("Keyword INTO is required for MATCH UPDATE"));
    };

    let uuid_str = chars
        .take_while(|c| c.is_alphanumeric() || c == &'-')
        .collect::<String>()
        .trim()
        .to_string();

    let uuid = Uuid::from_str(&uuid_str)
        .map_err(|e| format!("Couldn't create uuid from {}, Error: {:?}", uuid_str, e))?;

    match &entity_symbol.to_uppercase()[..] {
        "SET" => Ok(Wql::MatchUpdate(entity_name, entity_map, uuid, match_args?)),
        _ => Err("Couldn't parse MATCH UPDATE query".to_string()),
    }
}

fn evict(chars: &mut std::str::Chars) -> Result<Wql, String> {
    let info = chars
        .take_while(|c| c.is_alphanumeric() || c == &'-' || c == &'_')
        .collect::<String>()
        .trim()
        .to_string();

    let uuid = Uuid::from_str(&info);
    if uuid.is_err() {
        if info.chars().any(|c| c == '-') {
            return Err("Entity name cannot contain `-`".to_string());
        }
        Ok(Wql::Evict(info, None))
    } else {
        let from_symbol = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| !c.is_whitespace())
            .collect::<String>()
            .trim()
            .to_string();

        if from_symbol.to_uppercase() != "FROM" {
            return Err(String::from("Keyword FROM is required to EVICT an UUID"));
        }
        let name = chars
            .take_while(|c| c.is_alphanumeric() || c == &'_')
            .collect::<String>()
            .trim()
            .to_string();

        if name.is_empty() {
            return Err(String::from("Entity name is required for EVICT"));
        }

        Ok(Wql::Evict(name, uuid.ok()))
    }
}
