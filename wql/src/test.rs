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
            Wql::CreateEntity(String::from("entity"), Vec::new(), Vec::new())
        );
    }

    #[test]
    fn create_entity_with_uniques() {
        let wql = Wql::from_str("CREATE ENTITY entity UNIQUES #{name, ssn,something,}");

        assert_eq!(
            wql.unwrap(),
            Wql::CreateEntity(
                String::from("entity"),
                vec![
                    "name".to_string(),
                    "ssn".to_string(),
                    "something".to_string()
                ],
                Vec::new()
            )
        );
    }

    #[test]
    fn create_entity_with_encrypt() {
        let wql = Wql::from_str("CREATE ENTITY entity ENCRYPT #{name, ssn,something,}");

        assert_eq!(
            wql.unwrap(),
            Wql::CreateEntity(
                String::from("entity"),
                Vec::new(),
                vec![
                    "name".to_string(),
                    "ssn".to_string(),
                    "something".to_string()
                ],
            )
        );
    }

    #[test]
    fn create_entity_with_encrypt_and_uniques() {
        let wql = Wql::from_str(
            "CREATE ENTITY entity ENCRYPT #{password,something,} UNIQUES #{name, ssn,}",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::CreateEntity(
                String::from("entity"),
                vec!["name".to_string(), "ssn".to_string(),],
                vec!["password".to_string(), "something".to_string()],
            )
        );
    }

    #[test]
    fn create_uniques_in_encrypt() {
        let wql = Wql::from_str(
            "CREATE ENTITY entity ENCRYPT #{password,something,} UNIQUES #{name, something,}",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Encrypted arguments cannot be set to UNIQUE"))
        );
    }

    #[test]
    fn create_encrypts_in_uniques() {
        let wql = Wql::from_str(
            "CREATE ENTITY entity UNIQUES #{name, something,} ENCRYPT #{password,something,}",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Encrypted arguments cannot be set to UNIQUE"))
        );
    }

    #[test]
    fn create_entity_with_uniques_and_encrypt() {
        let wql = Wql::from_str(
            "CREATE ENTITY entity UNIQUES #{name, ssn,} ENCRYPT #{password,something,}",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::CreateEntity(
                String::from("entity"),
                vec!["name".to_string(), "ssn".to_string(),],
                vec!["password".to_string(), "something".to_string()],
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
            Wql::Insert("my_entity".to_string(), hashmap(), None)
        );
    }

    #[test]
    fn insert_precise() {
        let wql = Wql::from_str(
            "INSERT {
            a: 98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673P,
        } INTO my_entity",
        );

        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Precise("98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673".to_string()));

        assert_eq!(wql.unwrap(), Wql::Insert("my_entity".to_string(), hm, None));
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
            " MATCH ALL(a == 1, b >= 3, c != \"hello\", d < 7,)
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

#[cfg(test)]
mod evict {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn evict_entity() {
        let wql = Wql::from_str("EVICT my_entity");

        assert_eq!(wql.unwrap(), Wql::Evict(String::from("my_entity"), None));
    }

    #[test]
    fn evict_entity_with_dash() {
        let wql = Wql::from_str("EVICT my-entity");

        assert_eq!(
            wql.err(),
            Some(String::from("Entity name cannot contain `-`"))
        );
    }

    #[test]
    fn evict_entity_from_id() {
        let wql = Wql::from_str("EVICT d6ca73c0-41ff-4975-8a60-fc4a061ce536 FROM my_entity");

        assert_eq!(
            wql.unwrap(),
            Wql::Evict(
                String::from("my_entity"),
                Uuid::from_str("d6ca73c0-41ff-4975-8a60-fc4a061ce536").ok()
            )
        );
    }

    #[test]
    fn evict_entity_without_from() {
        let wql = Wql::from_str("EVICT d6ca73c0-41ff-4975-8a60-fc4a061ce536 my_entity");

        assert_eq!(
            wql.err(),
            Some(String::from("Keyword FROM is required to EVICT an UUID"))
        );
    }

    #[test]
    fn evict_entity_without_entity_name() {
        let wql = Wql::from_str("EVICT d6ca73c0-41ff-4975-8a60-fc4a061ce536 FROM");

        assert_eq!(
            wql.err(),
            Some(String::from("Entity name is required for EVICT"))
        );
    }
}

#[cfg(test)]
mod test_data_sructures {
    use super::*;

    #[test]
    fn insert_vec() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
            b: [12.3, 34, \"hello\",]
        } INTO my_entity",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::Insert("my_entity".to_string(), hashmap(), None)
        );
    }

    #[test]
    fn insert_time() {
        use chrono::{DateTime, Utc};
        let wql = Wql::from_str(
            "INSERT {
            time: 2014-11-28T12:00:09Z,
        } INTO my_entity",
        );

        let hm: HashMap<String, Types> = vec![(
            "time".to_string(),
            Types::DateTime("2014-11-28T12:00:09Z".parse::<DateTime<Utc>>().unwrap()),
        )]
        .iter()
        .cloned()
        .collect();
        assert_eq!(wql.unwrap(), Wql::Insert("my_entity".to_string(), hm, None));
    }

    #[test]
    fn insert_vec_in_vec() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
            b: [12.3, 34, [\"hello\"]]
        } INTO my_entity",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::Insert("my_entity".to_string(), hashmap2(), None)
        );
    }

    #[test]
    fn insert_vec_err() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
            b: [12.3, 34, \"hello\", nkjsld,]
        } INTO my_entity",
        );

        assert_eq!(
            wql.err(),
            Some(String::from("Value Type could not be created from nkjsld"))
        );
    }

    #[test]
    fn insert_with_err() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
            b: [12.3, 34, \"hello\",]
        } INTO my_entity
        ID 555555-5555-444444",
        );

        assert_eq!(
            wql.err(),
            Some(String::from(
                "Keyword WITH is required for INSERT with Uuid"
            ))
        );
    }

    #[test]
    fn insert_vec_with_map() {
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
            b: { a: 12.3, b: 34, }
        } INTO my_entity",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::Insert("my_entity".to_string(), hashmap3(), None)
        );
    }

    #[test]
    fn insert_vec_with_map_and_id() {
        let uuid = Uuid::parse_str("13ca62fc-241b-4af6-87c3-0ae4015f9967").ok();
        let wql = Wql::from_str(
            "INSERT {
            a: 123,
            b: { a: 12.3, b: 34, }
        } INTO my_entity
          WITH 13ca62fc-241b-4af6-87c3-0ae4015f9967",
        );

        assert_eq!(
            wql.unwrap(),
            Wql::Insert("my_entity".to_string(), hashmap3(), uuid)
        );
    }

    fn hashmap() -> HashMap<String, Types> {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Integer(123));
        hm.insert(
            "b".to_string(),
            Types::Vector(vec![
                Types::Float(12.3),
                Types::Integer(34),
                Types::String("hello".to_string()),
            ]),
        );
        hm
    }

    fn hashmap2() -> HashMap<String, Types> {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Integer(123));
        hm.insert(
            "b".to_string(),
            Types::Vector(vec![
                Types::Float(12.3),
                Types::Integer(34),
                Types::Vector(vec![Types::String("hello".to_string())]),
            ]),
        );
        hm
    }

    fn hashmap3() -> HashMap<String, Types> {
        let mut inner_map = HashMap::new();
        inner_map.insert("a".to_string(), Types::Float(12.3));
        inner_map.insert("b".to_string(), Types::Integer(34));
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Integer(123));
        hm.insert("b".to_string(), Types::Map(inner_map));
        hm
    }
}

#[cfg(test)]
mod check {
    use std::collections::HashMap;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn check_encrypt_values() {
        let wql = Wql::from_str(
            "CHECK {
            ssn: 123,
            pswd: \"my-password\"
        } FROM my_entity ID d6ca73c0-41ff-4975-8a60-fc4a061ce536",
        );

        let uuid = Uuid::from_str("d6ca73c0-41ff-4975-8a60-fc4a061ce536").unwrap();

        assert_eq!(
            wql.unwrap(),
            Wql::CheckValue("my_entity".to_string(), uuid, hashmap())
        );
    }

    fn hashmap() -> HashMap<String, String> {
        let mut hm = HashMap::new();
        hm.insert("ssn".to_string(), "123".to_string());
        hm.insert("pswd".to_string(), "my-password".to_string());
        hm
    }
}

#[cfg(test)]
mod test_where {
    use super::*;

    #[test]
    fn where_ok() {
        let query = "Select * FROM my_entity WherE {
            (in ?id 32434 45345 345346436),
            (between ?age 30 35),
        }";
        let wql = Wql::from_str(query);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![
                    Clause::ComplexComparisonFunctions(
                        where_clause::Function::In,
                        "?id".to_string(),
                        vec![
                            Types::Integer(32434),
                            Types::Integer(45345),
                            Types::Integer(345346436),
                        ]
                    ),
                    Clause::ComplexComparisonFunctions(
                        where_clause::Function::Between,
                        "?age".to_string(),
                        vec![Types::Integer(30), Types::Integer(35),]
                    )
                ],
                HashMap::new()
            )
        )
    }

    #[test]
    fn or_clause() {
        let query = "Select * FROM my_entity WherE {
            ?* my_entity:a ?a,
            ?* my_entity:c ?c,
            (== ?a 123),
            (or
                (>= c 4300.0)
                (< c 6.9)
            ),
        }";
        let wql = Wql::from_str(query);

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![
                    Clause::ValueAttribution(
                        "my_entity".to_string(),
                        "a".to_string(),
                        Value("?a".to_string())
                    ),
                    Clause::ValueAttribution(
                        "my_entity".to_string(),
                        "c".to_string(),
                        Value("?c".to_string())
                    ),
                    Clause::SimpleComparisonFunction(
                        Function::Eq,
                        "?a".to_string(),
                        Types::Integer(123)
                    ),
                    Clause::Or(
                        Function::Or,
                        vec![
                            Clause::SimpleComparisonFunction(
                                Function::GEq,
                                "c".to_string(),
                                Types::Float(4300.0)
                            ),
                            Clause::SimpleComparisonFunction(
                                Function::L,
                                "c".to_string(),
                                Types::Float(6.9)
                            )
                        ]
                    )
                ],
                HashMap::new()
            )
        )
    }

    #[test]
    fn select_where_groupby() {
        let query = "Select * FROM my_entity WHERE {
            ?* my_entity:name \"julia\",
            ?* my_entity:id 349875325,
        } GROUP BY amazing_key";
        let wql = Wql::from_str(query);
        let hm: HashMap<String, Algebra> = vec![(
            "GROUP".to_string(),
            Algebra::GroupBy(String::from("amazing_key")),
        )]
        .iter()
        .cloned()
        .collect();

        assert_eq!(
            wql.unwrap(),
            Wql::SelectWhere(
                "my_entity".to_string(),
                ToSelect::All,
                vec![
                    Clause::ContainsKeyValue(
                        "my_entity".to_string(),
                        "name".to_string(),
                        Types::String("julia".to_string())
                    ),
                    Clause::ContainsKeyValue(
                        "my_entity".to_string(),
                        "id".to_string(),
                        Types::Integer(349875325)
                    ),
                ],
                hm
            )
        )
    }
}

#[cfg(test)]
mod diff_intersect {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn intersect_key() {
        let f_uuid = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").ok();
        let s_uuid = Uuid::from_str("49dab8cf-2df2-474d-6fd1-c596c0bb8a00").ok();
        let query = "INTERSECT KEY SelEct * FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 | SelEct * FROM my_entity ID 49dab8cf-2df2-474d-6fd1-c596c0bb8a00";
        let wql = Wql::from_str(query);
        assert_eq!(
            wql.unwrap(),
            Wql::RelationQuery(
                vec![
                    Wql::Select(
                        "my_entity".to_string(),
                        ToSelect::All,
                        f_uuid,
                        HashMap::new()
                    ),
                    Wql::Select(
                        "my_entity".to_string(),
                        ToSelect::All,
                        s_uuid,
                        HashMap::new()
                    ),
                ],
                Relation::Intersect,
                RelationType::Key
            )
        );
    }

    #[test]
    fn diff_key_value() {
        let f_uuid = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").ok();
        let s_uuid = Uuid::from_str("49dab8cf-2df2-474d-6fd1-c596c0bb8a00").ok();
        let query = "DIFFERENCE KEY-VALUE SelEct * FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 | SelEct * FROM my_entity ID 49dab8cf-2df2-474d-6fd1-c596c0bb8a00 WHEN AT 2020-01-01T00:00:00Z";
        let wql = Wql::from_str(query);
        assert_eq!(
            wql.unwrap(),
            Wql::RelationQuery(
                vec![
                    Wql::Select(
                        "my_entity".to_string(),
                        ToSelect::All,
                        f_uuid,
                        HashMap::new()
                    ),
                    Wql::SelectWhen(
                        "my_entity".to_string(),
                        ToSelect::All,
                        s_uuid,
                        "2020-01-01T00:00:00Z".to_string()
                    ),
                ],
                Relation::Difference,
                RelationType::KeyValue
            )
        );
    }

    #[test]
    fn union_key() {
        let f_uuid = Uuid::from_str("2df2b8cf-49da-474d-8a00-c596c0bb6fd1").ok();
        let s_uuid = Uuid::from_str("49dab8cf-2df2-474d-6fd1-c596c0bb8a00").ok();
        let query = "UNION KEY SelEct * FROM my_entity ID 2df2b8cf-49da-474d-8a00-c596c0bb6fd1 | SelEct * FROM my_entity ID 49dab8cf-2df2-474d-6fd1-c596c0bb8a00 WHEN AT 2020-01-01T00:00:00Z";
        let wql = Wql::from_str(query);
        assert_eq!(
            wql.unwrap(),
            Wql::RelationQuery(
                vec![
                    Wql::Select(
                        "my_entity".to_string(),
                        ToSelect::All,
                        f_uuid,
                        HashMap::new()
                    ),
                    Wql::SelectWhen(
                        "my_entity".to_string(),
                        ToSelect::All,
                        s_uuid,
                        "2020-01-01T00:00:00Z".to_string()
                    ),
                ],
                Relation::Union,
                RelationType::Key
            )
        );
    }
}
