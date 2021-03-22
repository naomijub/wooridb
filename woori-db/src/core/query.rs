use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet},
};

use rayon::prelude::*;
use uuid::Uuid;
use wql::{Algebra, Types};

use crate::{
    model::error::Error,
    schemas::query::{CountResponse, Response as QueryResponse},
};

use super::pretty_config_inner;

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
        let key = if k.starts_with("NIL(") {
            String::from(&k[4..k.len() - 1])
        } else {
            k.to_owned()
        };

        let mut set: HashSet<String> = HashSet::new();
        let mut new_states: BTreeMap<Uuid, HashMap<String, Types>> = BTreeMap::new();
        for (id, state) in states {
            let k_value = state.get(&key);

            if k.starts_with("NIL(")
                && k_value.is_some()
                && k_value != Some(&Types::Nil)
                && !set.contains(&format!("{:?}", k_value.unwrap()))
            {
                set.insert(format!("{:?}", k_value.unwrap()));
                new_states.insert(id.to_owned(), state.to_owned());
            } else if !k.starts_with("NIL(")
                && !set.contains(&format!("{:?}", state.get(k).unwrap_or(&Types::Nil)))
            {
                set.insert(format!("{:?}", state.get(k).unwrap_or(&Types::Nil)));
                new_states.insert(id, state);
            }
        }
        new_states
    } else {
        states
    }
}

pub(crate) fn dedup_option_states(
    states: BTreeMap<Uuid, Option<HashMap<String, Types>>>,
    functions: &HashMap<String, wql::Algebra>,
) -> BTreeMap<Uuid, Option<HashMap<String, Types>>> {
    let dedup = functions.get("DEDUP");
    if let Some(Algebra::Dedup(k)) = dedup {
        let key = if k.starts_with("NIL(") {
            String::from(&k[4..k.len() - 1])
        } else {
            k.to_owned()
        };

        let mut set: HashSet<String> = HashSet::new();
        let mut new_states: BTreeMap<Uuid, Option<HashMap<String, Types>>> = BTreeMap::new();
        for (id, state) in states.iter().filter(|(_, s)| s.is_some()) {
            let some_state = state.clone().unwrap();
            let k_value = some_state.get(&key);

            if k.starts_with("NIL(")
                && k_value.is_some()
                && k_value != Some(&Types::Nil)
                && !set.contains("")
            {
                set.insert(format!("{:?}", k_value.unwrap()));
                new_states.insert(id.to_owned(), state.to_owned());
            } else if !set.contains(&format!("{:?}", k_value.unwrap_or(&Types::Nil))) {
                set.insert(format!("{:?}", k_value.unwrap_or(&Types::Nil)));
                new_states.insert(id.to_owned(), state.to_owned());
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
) -> Result<QueryResponse, Error> {
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
            Ok(CountResponse::new(
                size,
                ron::ser::to_string_pretty(&states, pretty_config_inner())?,
            )
            .into())
        } else {
            Ok(states.into())
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

                Ok(group_states.into())
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
                Ok(group_states.into())
            }
        } else {
            if should_count {
                let size = groups.keys().len();
                Ok(CountResponse::new(
                    size,
                    ron::ser::to_string_pretty(&groups, pretty_config_inner())?,
                )
                .into())
            } else {
                Ok(groups.into())
            }
        }
    } else {
        if should_count {
            let size = states.keys().len();
            Ok(CountResponse::new(
                size,
                ron::ser::to_string_pretty(&states, pretty_config_inner())?,
            )
            .into())
        } else {
            Ok(states.into())
        }
    }
}

pub(crate) fn get_result_after_manipulation_for_options(
    states: BTreeMap<Uuid, Option<HashMap<String, Types>>>,
    functions: HashMap<String, wql::Algebra>,
    should_count: bool,
) -> Result<QueryResponse, Error> {
    if let (Some(Algebra::OrderBy(k, ord)), None) = (functions.get("ORDER"), functions.get("GROUP"))
    {
        let states = states
            .into_par_iter()
            .map(|(id, state)| (id, state))
            .collect::<Vec<(Uuid, Option<HashMap<String, Types>>)>>();
        let mut states = states
            .into_par_iter()
            .filter(|(_, s)| s.is_some())
            .map(|(id, s)| (id, s.unwrap()))
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
            Ok(CountResponse::new(
                size,
                ron::ser::to_string_pretty(&states, pretty_config_inner())?,
            )
            .into())
        } else {
            Ok(states.into())
        }
    } else if let Some(Algebra::GroupBy(k)) = functions.get("GROUP") {
        let mut groups: HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>> =
            HashMap::new();
        for (id, state) in states {
            if let Some(s) = state {
                let key = s.get(k).unwrap_or(&Types::Nil);

                let g = groups
                    .entry(format!("{:?}", key))
                    .or_insert(BTreeMap::new());
                (*g).insert(id, Some(s));
            } else {
                let key = &Types::Nil;

                let g = groups
                    .entry(format!("{:?}", key))
                    .or_insert(BTreeMap::new());
                (*g).insert(id, None);
            }
        }
        if let Some(Algebra::OrderBy(k, ord)) = functions.get("ORDER") {
            let mut group_states = groups
                .into_par_iter()
                .map(|(key, states)| {
                    (
                        key,
                        states
                            .into_iter()
                            .filter(|(_, state)| state.is_some())
                            .map(|(id, state)| (id, state.unwrap()))
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

                Ok(group_states.into())
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
                Ok(group_states.into())
            }
        } else {
            if should_count {
                let size = groups.keys().len();
                Ok(CountResponse::new(
                    size,
                    ron::ser::to_string_pretty(&groups, pretty_config_inner())?,
                )
                .into())
            } else {
                Ok(groups.into())
            }
        }
    } else {
        if should_count {
            let size = states.keys().len();
            Ok(CountResponse::new(
                size,
                ron::ser::to_string_pretty(&states, pretty_config_inner())?,
            )
            .into())
        } else {
            Ok(states.into())
        }
    }
}
