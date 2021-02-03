use super::*;

pub(crate) fn read_match_args(chars: &mut std::str::Chars) -> Result<Vec<MatchCondition>, String> {
    let base = chars
        .skip_while(|c| c == &'(' || c.is_whitespace())
        .take_while(|c| c != &')')
        .collect::<String>()
        .trim()
        .to_string();
    let mut conditions: Vec<MatchCondition> = Vec::new();
    base.split(",")
        .map(|l| {
            let k = l
                .split(" ")
                .filter(|f| !f.is_empty())
                .collect::<Vec<&str>>();
            let mut c = k[2].chars();
            match k[1] {
                "==" => Ok(MatchCondition::Eq(
                    k[0].to_string(),
                    parse_value(
                        c.next()
                            .ok_or_else(|| String::from("Not able to parse match argument"))?,
                        &mut c,
                    )?,
                )),
                "!=" => Ok(MatchCondition::NotEq(
                    k[0].to_string(),
                    parse_value(
                        c.next()
                            .ok_or_else(|| String::from("Not able to parse match argument"))?,
                        &mut c,
                    )?,
                )),
                ">=" => Ok(MatchCondition::GEq(
                    k[0].to_string(),
                    parse_value(
                        c.next()
                            .ok_or_else(|| String::from("Not able to parse match argument"))?,
                        &mut c,
                    )?,
                )),
                "<=" => Ok(MatchCondition::LEq(
                    k[0].to_string(),
                    parse_value(
                        c.next()
                            .ok_or_else(|| String::from("Not able to parse match argument"))?,
                        &mut c,
                    )?,
                )),
                ">" => Ok(MatchCondition::G(
                    k[0].to_string(),
                    parse_value(
                        c.next()
                            .ok_or_else(|| String::from("Not able to parse match argument"))?,
                        &mut c,
                    )?,
                )),
                "<" => Ok(MatchCondition::L(
                    k[0].to_string(),
                    parse_value(
                        c.next()
                            .ok_or_else(|| String::from("Not able to parse match argument"))?,
                        &mut c,
                    )?,
                )),
                _ => Err(String::from("Unidentified Match Condition")),
            }
        })
        .try_for_each(|e: Result<MatchCondition, String>| {
            conditions.push(e?);
            Ok::<(), String>(())
        })?;

    Ok(conditions)
}

pub(crate) fn read_map(chars: &mut std::str::Chars) -> Result<HashMap<String, Types>, String> {
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

pub(crate) fn parse_key(c: char, chars: &mut std::str::Chars) -> String {
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

pub(crate) fn read_str(chars: &mut std::str::Chars) -> Result<Types, String> {
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
