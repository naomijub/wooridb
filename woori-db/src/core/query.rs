use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet},
};

use rayon::prelude::*;
use uuid::Uuid;
use wql::{Algebra, Types};

use crate::{model::error::Error, schemas::query::CountResponse};

use super::pretty_config_output;

pub(crate) fn get_limit_offset_count(
    functions: &HashMap<String, wql::Algebra>,
) -> (usize, usize, bool) {
    let limit = if let Some(Algebra::Limit(l)) = functions.get("LIMIT") {
        *l
    } else {
        usize::MAX
    };
    let offset = if let Some(Algebra::Offset(o)) = functions.get("OFFSET") {
        *o
    } else {
        0
    };
    let count = if let Some(Algebra::Count) = functions.get("COUNT") {
        true
    } else {
        false
    };

    (limit, offset, count)
}

pub(crate) fn dedup_states(
    states: BTreeMap<Uuid, HashMap<String, Types>>,
    functions: &HashMap<String, wql::Algebra>,
) -> BTreeMap<Uuid, HashMap<String, Types>> {
    if let Some(Algebra::Dedup(k)) = functions.get("DEDUP") {
        let mut set: HashSet<String> = HashSet::new();
        let mut new_states: BTreeMap<Uuid, HashMap<String, Types>> = BTreeMap::new();
        for (id, state) in states {
            if !set.contains(&format!("{:?}", state.get(k).unwrap_or(&Types::Nil))) {
                set.insert(format!("{:?}", state.get(k).unwrap_or(&Types::Nil)));
                new_states.insert(id, state);
            }
        }
        new_states
    } else {
        states
    }
}

pub(crate) fn get_result_after_manipulation(
    states: BTreeMap<Uuid, HashMap<String, Types>>,
    functions: HashMap<String, wql::Algebra>,
    should_count: bool,
) -> Result<String, Error> {
    if let (Some(Algebra::OrderBy(k, ord)), None) = (functions.get("ORDER"), functions.get("GROUP"))
    {
        let mut states = states
            .into_par_iter()
            .map(|(id, state)| (id, state))
            .collect::<Vec<(Uuid, HashMap<String, Types>)>>();
        if ord == &wql::Order::Asc {
            states.sort_by(|a, b| {
                a.1.get(k)
                    .partial_cmp(&b.1.get(k))
                    .unwrap_or(Ordering::Less)
            });
        } else {
            states.sort_by(|a, b| {
                b.1.get(k)
                    .partial_cmp(&a.1.get(k))
                    .unwrap_or(Ordering::Less)
            });
        }
        if should_count {
            let size = states.len();
            CountResponse::to_response(
                size,
                ron::ser::to_string_pretty(&states, pretty_config_output())?,
            )
        } else {
            Ok(ron::ser::to_string_pretty(&states, pretty_config_output())?)
        }
    } else if let Some(Algebra::GroupBy(k)) = functions.get("GROUP") {
        let mut groups: HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>> = HashMap::new();
        for (id, state) in states {
            let key = state.get(k).unwrap_or(&Types::Nil);
            let g = groups
                .entry(format!("{:?}", key))
                .or_insert(BTreeMap::new());
            (*g).insert(id, state);
        }
        if let Some(Algebra::OrderBy(k, ord)) = functions.get("ORDER") {
            let mut group_states = groups
                .into_par_iter()
                .map(|(key, states)| {
                    (
                        key,
                        states
                            .into_iter()
                            .map(|(id, state)| (id, state))
                            .collect::<Vec<(Uuid, HashMap<String, Types>)>>(),
                    )
                })
                .collect::<HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>>();

            if ord == &wql::Order::Asc {
                let group_states = group_states
                    .iter_mut()
                    .map(|(key, states)| {
                        states.sort_by(|a, b| {
                            a.1.get(k)
                                .partial_cmp(&b.1.get(k))
                                .unwrap_or(Ordering::Less)
                        });
                        (key.to_owned(), states.to_owned())
                    })
                    .collect::<HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>>();

                Ok(ron::ser::to_string_pretty(
                    &group_states,
                    pretty_config_output(),
                )?)
            } else {
                let group_states = group_states
                    .iter_mut()
                    .map(|(key, states)| {
                        states.sort_by(|a, b| {
                            b.1.get(k)
                                .partial_cmp(&a.1.get(k))
                                .unwrap_or(Ordering::Less)
                        });
                        (key.to_owned(), states.to_owned())
                    })
                    .collect::<HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>>();
                Ok(ron::ser::to_string_pretty(
                    &group_states,
                    pretty_config_output(),
                )?)
            }
        } else {
            if should_count {
                let size = groups.keys().len();
                CountResponse::to_response(
                    size,
                    ron::ser::to_string_pretty(&groups, pretty_config_output())?,
                )
            } else {
                Ok(ron::ser::to_string_pretty(&groups, pretty_config_output())?)
            }
        }
    } else {
        if should_count {
            let size = states.keys().len();
            CountResponse::to_response(
                size,
                ron::ser::to_string_pretty(&states, pretty_config_output())?,
            )
        } else {
            Ok(ron::ser::to_string_pretty(&states, pretty_config_output())?)
        }
    }
}
