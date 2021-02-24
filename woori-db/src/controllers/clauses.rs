use std::collections::{BTreeMap, HashMap};

use futures::{future, stream, StreamExt};
use uuid::Uuid;
use wql::{Clause, ToSelect, Types, Value};

use crate::{
    actors::state::State,
    core::{pretty_config_output, registry::get_registries},
    model::{error::Error, DataExecutor, DataLocalContext, DataRegister},
};

pub async fn select_where(
    entity: String,
    args_to_select: ToSelect,
    clauses: Vec<Clause>,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let args_to_key = clauses
        .clone()
        .into_iter()
        .filter_map(|clause| {
            if let Clause::ValueAttribution(_, key, Value(arg)) = clause {
                Some((arg, key))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>();
    let registries = get_registries(&entity, &local_data)?;
    let states = generate_state(&registries, args_to_select, &actor).await?;
    let states = filter_where_clauses(states, args_to_key, &clauses).await;

    Ok(ron::ser::to_string_pretty(&states, pretty_config_output())?)
}

async fn filter_where_clauses(
    states: BTreeMap<Uuid, HashMap<String, Types>>,
    args_to_key: HashMap<String, String>,
    clauses: &[Clause],
) -> BTreeMap<Uuid, HashMap<String, Types>> {
    let default = String::new();
    stream::iter(states)
        .filter(|(_, state)| {
            future::ready(
                clauses
                    .iter()
                    .map(|clause| match clause {
                        Clause::ValueAttribution(_, _, _) => true,
                        Clause::Or(_, inner_clauses) => {
                            or_clauses(state, &args_to_key, inner_clauses)
                        }
                        Clause::ContainsKeyValue(_, key, value) => {
                            state.get(key).map_or(false, |v| value == v)
                        }
                        Clause::SimpleComparisonFunction(f, key, value) => {
                            let key = args_to_key.get(key).unwrap_or(&default);
                            state.get(key).map_or(false, |v| match f {
                                wql::Function::Eq => v == value,
                                wql::Function::NotEq => v != value,
                                wql::Function::GEq => v >= value,
                                wql::Function::G => v > value,
                                wql::Function::LEq => v <= value,
                                wql::Function::L => v < value,
                                wql::Function::Like => {
                                    if let (Types::String(content), Types::String(regex)) =
                                        (v, value)
                                    {
                                        let pattern = regex.replace("%", "");
                                        if regex.starts_with('%') && regex.ends_with('%') {
                                            content.contains(&pattern)
                                        } else if regex.starts_with('%') {
                                            content.ends_with(&pattern)
                                        } else if regex.ends_with('%') {
                                            content.starts_with(&pattern)
                                        } else {
                                            content.contains(&pattern)
                                        }
                                    } else {
                                        false
                                    }
                                }
                                _ => false,
                            })
                        }
                        Clause::ComplexComparisonFunctions(wql::Function::In, key, set) => {
                            let key = args_to_key.get(key).unwrap_or(&default);
                            state.get(key).map_or(false, |v| set.contains(v))
                        }
                        Clause::ComplexComparisonFunctions(
                            wql::Function::Between,
                            key,
                            start_end,
                        ) => {
                            let key = args_to_key.get(key).unwrap_or(&default);
                            state
                                .get(key)
                                .map_or(false, |v| v >= &start_end[0] && v <= &start_end[1])
                        }
                        _ => false,
                    })
                    .all(|f| f),
            )
        })
        .collect::<BTreeMap<Uuid, HashMap<String, Types>>>()
        .await
}

fn or_clauses(
    state: &HashMap<std::string::String, wql::Types>,
    args_to_key: &HashMap<String, String>,
    inner_clauses: &[Clause],
) -> bool {
    let default = String::new();
    inner_clauses
        .iter()
        .map(|clause| match clause {
            Clause::ValueAttribution(_, _, _) => true,
            Clause::Error => false,
            Clause::Or(_, or_inner_clauses) => or_clauses(state, &args_to_key, or_inner_clauses),
            Clause::ContainsKeyValue(_, key, value) => state.get(key).map_or(false, |v| value == v),
            Clause::SimpleComparisonFunction(f, key, value) => {
                state.get(key).map_or(false, |v| match f {
                    wql::Function::Eq => v == value,
                    wql::Function::NotEq => v != value,
                    wql::Function::GEq => v >= value,
                    wql::Function::G => v > value,
                    wql::Function::LEq => v <= value,
                    wql::Function::L => v < value,
                    wql::Function::Like => {
                        if let (Types::String(content), Types::String(regex)) = (v, value) {
                            if regex.starts_with('%') && regex.ends_with('%') {
                                content.contains(&regex[1..regex.len() - 1])
                            } else if regex.starts_with('%') {
                                content.ends_with(&regex[..regex.len() - 1])
                            } else if regex.ends_with('%') {
                                content.starts_with(&regex[1..])
                            } else {
                                content.contains(&regex[..])
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
            }
            Clause::ComplexComparisonFunctions(wql::Function::In, key, set) => {
                let key = args_to_key.get(key).unwrap_or(&default);
                state.get(key).map_or(false, |v| set.contains(v))
            }
            Clause::ComplexComparisonFunctions(wql::Function::Between, key, start_end) => {
                let key = args_to_key.get(key).unwrap_or(&default);
                state
                    .get(key)
                    .map_or(false, |v| v >= &start_end[0] && v <= &start_end[1])
            }
            _ => false,
        })
        .any(|f| f)
}

async fn generate_state(
    registries: &BTreeMap<Uuid, DataRegister>,
    args_to_select: ToSelect,
    actor: &DataExecutor,
) -> Result<BTreeMap<Uuid, HashMap<String, Types>>, Error> {
    let mut states: BTreeMap<Uuid, HashMap<String, Types>> = BTreeMap::new();
    for (uuid, regs) in registries {
        let content = actor.send(regs.to_owned()).await??;
        let state = actor
            .send(State(content))
            .await??
            .into_iter()
            .filter(|(_, v)| !v.is_hash());
        let filtered = if let ToSelect::Keys(ref keys) = args_to_select {
            state
                .filter(|(k, _)| keys.contains(k))
                .collect::<HashMap<String, Types>>()
        } else {
            state.collect::<HashMap<String, Types>>()
        };

        states.insert(uuid.to_owned(), filtered);
    }
    Ok(states)
}
