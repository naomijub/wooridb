use super::{read_map, read_match_args, FromStr, MatchCondition, Uuid, Wql};

pub(crate) fn read_symbol(a: char, chars: &mut std::str::Chars) -> Result<Wql, String> {
    let symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();

    match (a, &symbol.to_uppercase()[..]) {
        ('c', "REATE") | ('C', "REATE") => create_entity(chars),
        ('i', "NSERT") | ('I', "NSERT") => insert(chars),
        ('u', "PDATE") | ('U', "PDATE") => update(chars),
        ('d', "ELETE") | ('D', "ELETE") => delete(chars),
        ('m', "ATCH") | ('M', "ATCH") => match_update(chars),
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

    let unique_symbol = chars.take_while(|c| !c.is_whitespace()).collect::<String>();
    if unique_symbol.to_uppercase() == "UNIQUES" {
        let unique_names = chars
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| {
                c.is_alphanumeric() || c == &'_' || c == &',' || c.is_whitespace() || c != &';'
            })
            .collect::<String>()
            .trim()
            .to_string();

        let unique_vec = unique_names
            .split(',')
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

    if into_symbol.to_uppercase() != "INTO" {
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

    let uuid =
        Uuid::from_str(&uuid_str).map_err(|_| format!("Couldn't create uuid from {}", uuid_str))?;

    match &entity_symbol.to_uppercase()[..] {
        "SET" => Ok(Wql::MatchUpdate(entity_name, entity_map, uuid, match_args?)),
        _ => Err("Couldn't parse UPDATE query".to_string()),
    }
}
