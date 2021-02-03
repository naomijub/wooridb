use actix::prelude::*;
use std::collections::HashMap;
use wql::{MatchCondition, Types};

use crate::model::error::Error;
use crate::{actors::wql::Executor, model::DataRegister};

pub struct State(pub String);

impl Message for State {
    type Result = Result<HashMap<String, Types>, Error>;
}

impl Handler<State> for Executor {
    type Result = Result<HashMap<String, Types>, Error>;

    fn handle(&mut self, msg: State, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;

        let fractions = msg.0.split('|').collect::<Vec<&str>>();
        if fractions[0].eq("INSERT") {
            let state = fractions.last().unwrap().to_owned();
            let state = &state[..(state.len() - 1)];

            let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                Ok(x) => Ok(x),
                Err(_) => Err(Error::FailedToParseState),
            };
            resp
        } else if fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT") {
            let state = fractions[fractions.len() - 2];

            let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                Ok(x) => Ok(x),
                Err(_) => Err(Error::FailedToParseState),
            };
            resp
        } else {
            Err(Error::FailedToParseState)
        }
    }
}

pub struct PreviousRegistry(pub String);

impl Message for PreviousRegistry {
    type Result = Result<Option<DataRegister>, Error>;
}

impl Handler<PreviousRegistry> for Executor {
    type Result = Result<Option<DataRegister>, Error>;

    fn handle(&mut self, msg: PreviousRegistry, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;

        let fractions = msg.0.split('|').collect::<Vec<&str>>();
        if fractions[0].eq("INSERT") {
            Ok(None)
        } else if fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT") {
            let state = fractions.last().unwrap().to_owned();
            let state = &state[..(state.len() - 1)];

            let resp: Result<DataRegister, Error> = match from_str(state) {
                Ok(x) => Ok(x),
                Err(_) => Err(Error::FailedToParseRegistry),
            };
            Ok(Some(resp?))
        } else {
            Err(Error::FailedToParseRegistry)
        }
    }
}

pub struct MatchUpdate {
    pub conditions: MatchCondition,
    pub previous_state: HashMap<String, Types>,
}

impl Message for MatchUpdate {
    type Result = Result<(), Error>;
}

impl Handler<MatchUpdate> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: MatchUpdate, _: &mut Self::Context) -> Self::Result {
        match msg.conditions.clone() {
            MatchCondition::All(all) => {
                let conds = all
                    .iter()
                    .map(|cond| match cond.clone() {
                        MatchCondition::Eq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                msg.previous_state[&key] == val
                            } else {
                                false
                            }
                        }
                        MatchCondition::NotEq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                msg.previous_state[&key] != val
                            } else {
                                false
                            }
                        }
                        MatchCondition::GEq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                ge_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        MatchCondition::LEq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                le_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        MatchCondition::G(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                g_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        MatchCondition::L(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                l_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        _ => false,
                    })
                    .all(|c| c);
                match conds {
                    true => Ok(()),
                    false => Err(Error::FailedMatchCondition),
                }
            }
            MatchCondition::Any(any) => {
                let conds = any
                    .iter()
                    .map(|cond| match cond.clone() {
                        MatchCondition::Eq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                msg.previous_state[&key] == val
                            } else {
                                false
                            }
                        }
                        MatchCondition::NotEq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                msg.previous_state[&key] != val
                            } else {
                                false
                            }
                        }
                        MatchCondition::GEq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                ge_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        MatchCondition::LEq(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                le_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        MatchCondition::G(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                g_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        MatchCondition::L(key, val) => {
                            if msg.previous_state.get(&key).is_some() {
                                l_match_types(val, msg.previous_state[&key].clone())
                            } else {
                                false
                            }
                        }
                        _ => false,
                    })
                    .any(|c| c);
                match conds {
                    true => Ok(()),
                    false => Err(Error::FailedMatchCondition),
                }
            }
            _ => Err(Error::UnkwonCondition),
        }?;
        Ok(())
    }
}

fn ge_match_types(cond: Types, state: Types) -> bool {
    match (cond, state) {
        (Types::Integer(c), Types::Integer(s)) => s >= c,
        (Types::Float(c), Types::Float(s)) => s >= c,
        _ => false,
    }
}

fn g_match_types(cond: Types, state: Types) -> bool {
    match (cond, state) {
        (Types::Integer(c), Types::Integer(s)) => s > c,
        (Types::Float(c), Types::Float(s)) => s > c,
        _ => false,
    }
}

fn le_match_types(cond: Types, state: Types) -> bool {
    match (cond, state) {
        (Types::Integer(c), Types::Integer(s)) => s <= c,
        (Types::Float(c), Types::Float(s)) => s <= c,
        _ => false,
    }
}

fn l_match_types(cond: Types, state: Types) -> bool {
    match (cond, state) {
        (Types::Integer(c), Types::Integer(s)) => s < c,
        (Types::Float(c), Types::Float(s)) => s < c,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::actors::wql::Executor;

    #[actix_rt::test]
    async fn test_all_matches() {
        let actor = Executor::new().start();
        let conds = MatchCondition::All(vec![
            MatchCondition::Eq("e".to_string(), Types::String(String::from("hello"))),
            MatchCondition::NotEq("f".to_string(), Types::Boolean(false)),
            MatchCondition::GEq("a".to_string(), Types::Float(3f64)),
            MatchCondition::LEq("b".to_string(), Types::Integer(7isize)),
            MatchCondition::G("c".to_string(), Types::Float(3f64)),
            MatchCondition::L("d".to_string(), Types::Integer(7)),
        ]);

        let result = actor
            .send(MatchUpdate {
                conditions: conds,
                previous_state: previous_state(),
            })
            .await
            .unwrap();

        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_any_matches() {
        let actor = Executor::new().start();
        let conds = MatchCondition::Any(vec![
            MatchCondition::Eq("e".to_string(), Types::String(String::from("hellwo"))),
            MatchCondition::NotEq("f".to_string(), Types::Boolean(true)),
            MatchCondition::GEq("a".to_string(), Types::Float(34f64)),
            MatchCondition::LEq("b".to_string(), Types::Integer(7isize)),
            MatchCondition::G("c".to_string(), Types::Float(34f64)),
            MatchCondition::L("d".to_string(), Types::Integer(-7)),
        ]);

        let result = actor
            .send(MatchUpdate {
                conditions: conds,
                previous_state: previous_state(),
            })
            .await
            .unwrap();

        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_any_fail() {
        let actor = Executor::new().start();
        let conds = MatchCondition::Any(vec![
            MatchCondition::Eq("e".to_string(), Types::String(String::from("hellwo"))),
            MatchCondition::NotEq("f".to_string(), Types::Boolean(true)),
            MatchCondition::GEq("a".to_string(), Types::Float(34f64)),
            MatchCondition::LEq("b".to_string(), Types::Integer(-7isize)),
            MatchCondition::G("c".to_string(), Types::Float(34f64)),
            MatchCondition::L("d".to_string(), Types::Integer(-7)),
        ]);

        let result = actor
            .send(MatchUpdate {
                conditions: conds,
                previous_state: previous_state(),
            })
            .await
            .unwrap();

        assert!(result.is_err());
    }

    #[actix_rt::test]
    async fn test_all_fail() {
        let actor = Executor::new().start();
        let conds = MatchCondition::All(vec![
            MatchCondition::Eq("e".to_string(), Types::String(String::from("hello"))),
            MatchCondition::NotEq("f".to_string(), Types::Boolean(false)),
            MatchCondition::GEq("a".to_string(), Types::Float(3f64)),
            MatchCondition::LEq("b".to_string(), Types::Integer(-7isize)),
            MatchCondition::G("c".to_string(), Types::Float(3f64)),
            MatchCondition::L("d".to_string(), Types::Integer(7)),
        ]);

        let result = actor
            .send(MatchUpdate {
                conditions: conds,
                previous_state: previous_state(),
            })
            .await
            .unwrap();

        assert!(result.is_err());
    }

    fn previous_state() -> HashMap<String, Types> {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), Types::Float(4.5f64));
        hm.insert("b".to_string(), Types::Integer(4));
        hm.insert("c".to_string(), Types::Float(5.5f64));
        hm.insert("d".to_string(), Types::Integer(6));
        hm.insert("e".to_string(), Types::String(String::from("hello")));
        hm.insert("f".to_string(), Types::Boolean(true));

        hm
    }
}
