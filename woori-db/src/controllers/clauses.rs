use std::collections::{BTreeMap, HashMap};

use rayon::prelude::*;
use uuid::Uuid;
use wql::{Algebra, Clause, ToSelect, Types, Value};

use crate::{
    core::registry::get_registries,
    model::{error::Error, DataLocalContext, DataRegister},
    schemas::query::Response as QueryResponse,
};

use crate::core::query::{dedup_states, get_limit_offset_count, get_result_after_manipulation};

pub async fn select_where_controller(
    entity: String,
    args_to_select: ToSelect,
    clauses: Vec<Clause>,
    local_data: DataLocalContext,
    functions: HashMap<String, wql::Algebra>,
) -> Result<QueryResponse, Error> {
    let states = select_where(entity, args_to_select, clauses, local_data, &functions);
    let count = matches!(functions.get("COUNT"), Some(Algebra::Count));

    Ok(get_result_after_manipulation(
        states.await?,
        &functions,
        count,
    ))
}

pub async fn select_where(
    entity: String,
    args_to_select: ToSelect,
    clauses: Vec<Clause>,
    local_data: DataLocalContext,
    functions: &HashMap<String, wql::Algebra>,
) -> Result<BTreeMap<Uuid, HashMap<String, Types>>, Error> {
    let (limit, offset, _) = get_limit_offset_count(functions);
    let args_to_key = clauses
        .clone()
        .into_par_iter()
        .filter_map(|clause| {
            if let Clause::ValueAttribution(_, key, Value(arg)) = clause {
                Some((arg, key))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>();
    let registries = get_registries(&entity, &local_data)?;
    let states = generate_state(&registries, args_to_select).await?;
    let states = filter_where_clauses(states, args_to_key, &clauses)
        .await
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let states = dedup_states(states, &functions);
    Ok(states)
}

async fn filter_where_clauses(
    states: BTreeMap<Uuid, HashMap<String, Types>>,
    args_to_key: HashMap<String, String>,
    clauses: &[Clause],
) -> BTreeMap<Uuid, HashMap<String, Types>> {
    let default = String::new();
    let mut states = states.clone();
    for clause in clauses {
        match clause {
            Clause::ValueAttribution(_, _, _) => {}
            Clause::Or(_, inner_clauses) => {
                for (id, state) in states.clone() {
                    if !or_clauses(&state, &args_to_key, &inner_clauses) {
                        states.remove(&id);
                    }
                }
            }
            Clause::ContainsKeyValue(_, key, value) => {
                for (id, state) in states.clone() {
                    if !state.get(key).map_or(false, |v| value == v) {
                        states.remove(&id);
                    }
                }
            }
            Clause::SimpleComparisonFunction(f, key, value) => {
                let key = args_to_key.get(key).unwrap_or(&default);
                for (id, state) in states.clone() {
                    state.get(key).map(|v| match f {
                        wql::Function::Eq => {
                            if !(v == value) {
                                states.remove(&id);
                            }
                        }
                        wql::Function::NotEq => {
                            if !(v != value) {
                                states.remove(&id);
                            }
                        }
                        wql::Function::GEq => {
                            if !(v >= &value) {
                                states.remove(&id);
                            }
                        }
                        wql::Function::G => {
                            if !(v > &value) {
                                states.remove(&id);
                            }
                        }
                        wql::Function::LEq => {
                            if !(v <= &value) {
                                states.remove(&id);
                            }
                        }
                        wql::Function::L => {
                            if !(v < &value) {
                                states.remove(&id);
                            }
                        }
                        wql::Function::Like => {
                            if let (Types::String(content), Types::String(regex)) = (v, value) {
                                let pattern = regex.replace("%", "");

                                if (regex.starts_with('%')
                                    && regex.ends_with('%')
                                    && content.contains(&pattern))
                                    || (regex.starts_with('%') && content.ends_with(&pattern))
                                    || (regex.ends_with('%') && content.starts_with(&pattern))
                                    || content.contains(&pattern)
                                {
                                    ()
                                } else {
                                    states.remove(&id);
                                }
                            } else {
                                states.remove(&id);
                            }
                        }
                        _ => {}
                    });
                }
            }
            Clause::ComplexComparisonFunctions(wql::Function::In, key, set) => {
                let key = args_to_key.get(key).unwrap_or(&default);
                for (id, state) in states.clone() {
                    if !state.get(key).map_or(false, |v| set.contains(v)) {
                        states.remove(&id);
                    }
                }
            }
            Clause::ComplexComparisonFunctions(wql::Function::Between, key, start_end) => {
                let key = args_to_key.get(key).unwrap_or(&default);
                for (id, state) in states.clone() {
                    if !state
                        .get(key)
                        .map_or(false, |v| v >= &start_end[0] && v <= &start_end[1])
                    {
                        states.remove(&id);
                    }
                }
            }
            _ => (),
        }
    }

    states
}

fn or_clauses(
    state: &HashMap<std::string::String, wql::Types>,
    args_to_key: &HashMap<String, String>,
    inner_clauses: &[Clause],
) -> bool {
    let default = String::new();
    inner_clauses
        .par_iter()
        .map(|clause| match clause {
            Clause::ValueAttribution(_, _, _) => true,
            Clause::Or(_, or_inner_clauses) => or_clauses(state, &args_to_key, or_inner_clauses),
            Clause::ContainsKeyValue(_, key, value) => {
                let key = args_to_key.get(key).unwrap_or(&default);
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
                        if let (Types::String(content), Types::String(regex)) = (v, value) {
                            if regex.starts_with('%') && regex.ends_with('%') {
                                let regex = regex.replace("%", "");
                                content.contains(&regex)
                            } else if regex.starts_with('%') {
                                let regex = regex.replace("%", "");
                                content.ends_with(&regex)
                            } else if regex.ends_with('%') {
                                let regex = regex.replace("%", "");
                                content.starts_with(&regex)
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
    registries: &BTreeMap<Uuid, (DataRegister, HashMap<String, Types>)>,
    args_to_select: ToSelect,
) -> Result<BTreeMap<Uuid, HashMap<String, Types>>, Error> {
    let mut states: BTreeMap<Uuid, HashMap<String, Types>> = BTreeMap::new();
    for (uuid, (_, state)) in registries {
        let state = state
            .into_par_iter()
            .filter(|(_, v)| !v.is_hash())
            .map(|(k, v)| (k.to_owned(), v.to_owned()));
        let filtered = if let ToSelect::Keys(ref keys) = args_to_select {
            state
                .into_par_iter()
                .filter(|(k, _)| keys.contains(k))
                .collect::<HashMap<String, Types>>()
        } else {
            state.collect::<HashMap<String, Types>>()
        };

        states.insert(uuid.to_owned(), filtered);
    }
    Ok(states)
}
