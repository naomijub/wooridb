use super::*;
use std::collections::HashMap;
use uuid::Uuid;

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

        assert!(wql
            .err()
            .unwrap()
            .starts_with("Couldn\'t create uuid from Some-crazy-id"));
    }
}

#[cfg(test)]
mod test_match {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_read_match_args() {
        let mut args = "(a == 1, b != 2, c > 3, d >= 4, e < 5, f <= 6)".chars();
        let actual = read_match_args(&mut args).unwrap();
        let expected = vec![
            MatchCondition::Eq("a".to_string(), Types::Integer(1)),
            MatchCondition::NotEq("b".to_string(), Types::Integer(2)),
            MatchCondition::G("c".to_string(), Types::Integer(3)),
            MatchCondition::GEq("d".to_string(), Types::Integer(4)),
            MatchCondition::L("e".to_string(), Types::Integer(5)),
            MatchCondition::LEq("f".to_string(), Types::Integer(6)),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn match_update_set_entity() {
        let wql = Wql::from_str(
            " MATCH ALL(a == 1, b >= 3, c != \"hello\", d < 7)
        UPDATE this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::MatchUpdate(
                "this_entity".to_string(),
                hashmap(),
                Uuid::from_str("d6ca73c0-41ff-4975-8a60-fc4a061ce536").unwrap(),
                MatchCondition::All(vec![
                    MatchCondition::Eq("a".to_string(), Types::Integer(1)),
                    MatchCondition::GEq("b".to_string(), Types::Integer(3)),
                    MatchCondition::NotEq("c".to_string(), Types::String("hello".to_string())),
                    MatchCondition::L("d".to_string(), Types::Integer(7)),
                ])
            )
        );
    }

    #[test]
    fn match_update_missing_logical_arg() {
        let wql = Wql::from_str(
            " MATCH (a == 1, b >= 3, c != \"hello\", d < 7)
        UPDATE this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err().unwrap(),
            String::from("MATCH requires ALL or ANY symbols")
        );
    }

    #[test]
    fn match_update_missing_update_key() {
        let wql = Wql::from_str(
            " MATCH Any(a == 1, b >= 3, c != \"hello\", d < 7)
        this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err().unwrap(),
            String::from("UPDATE keyword is required for MATCH UPDATE")
        );
    }

    #[test]
    fn match_update_missing_entity_name() {
        let wql = Wql::from_str(
            " MATCH All(a == 1, b >= 3, c != \"hello\", d < 7)
        UPDATE 
        SET {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err().unwrap(),
            String::from("Entity name is required for MATCH UPDATE")
        );
    }

    #[test]
    fn match_update_missing_set() {
        let wql = Wql::from_str(
            " MATCH All(a == 1, b >= 3, c != \"hello\", d < 7)
        UPDATE this_entity 
        {
            a: 123,
            g: NiL
        } 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err().unwrap(),
            String::from("MATCH UPDATE type is required after entity. Keyword is SET")
        );
    }

    #[test]
    fn match_update_missing_content() {
        let wql = Wql::from_str(
            " MATCH All(a == 1, b >= 3, c != \"hello\", d < 7)
        UPDATE this_entity 
        SET 
        INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err().unwrap(),
            String::from("Entity map should start with `{` and end with `}`")
        );
    }

    #[test]
    fn match_update_missing_into() {
        let wql = Wql::from_str(
            " MATCH All(a == 1, b >= 3, c != \"hello\", d < 7)
        UPDATE this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        assert_eq!(
            wql.err().unwrap(),
            String::from("Keyword INTO is required for MATCH UPDATE")
        );
    }

    #[test]
    fn match_update_missing_id() {
        let wql = Wql::from_str(
            " MATCH All(a == 1, b >= 3, c != \"hello\", d < 7)
        UPDATE this_entity 
        SET {
            a: 123,
            g: NiL
        } 
        INTO",
        );

        assert!(wql
            .err()
            .unwrap()
            .starts_with("Couldn\'t create uuid from "));
    }

    fn hashmap() -> Entity {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Integer(123));
        hm.insert("g".to_string(), Types::Nil);
        hm
    }
}
