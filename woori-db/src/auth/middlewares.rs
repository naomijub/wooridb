use crate::repository::local::{SessionContext, SessionInfo};
use actix_web::{dev::ServiceRequest, web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use super::schemas::Role;

pub async fn wql_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    if req.path().starts_with("/wql/tx") {
        let allow = req
            .app_data::<web::Data<Arc<Mutex<SessionContext>>>>()
            .and_then(|db| {
                validate_token(
                    &db,
                    Some(credentials.token()),
                    vec![Role::Write, Role::User],
                )
            });

        if let Some(true) = allow {
            Ok(req)
        } else {
            Err(crate::model::error::Error::AuthBadRequest.into())
        }
    } else if req.path().starts_with("/wql/query") {
        let allow = req
            .app_data::<web::Data<Arc<Mutex<SessionContext>>>>()
            .and_then(|db| {
                validate_token(&db, Some(credentials.token()), vec![Role::Read, Role::User])
            });

        if let Some(true) = allow {
            Ok(req)
        } else {
            Err(crate::model::error::Error::AuthBadRequest.into())
        }
    } else {
        Ok(req)
    }
}

pub async fn history_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    println!("{:?}", req.path());
    if req.path().starts_with("/entity-history") {
        let allow = req
            .app_data::<web::Data<Arc<Mutex<SessionContext>>>>()
            .and_then(|db| {
                validate_token(
                    &db,
                    Some(credentials.token()),
                    vec![Role::History, Role::User],
                )
            });

        if let Some(true) = allow {
            Ok(req)
        } else {
            Err(crate::model::error::Error::AuthBadRequest.into())
        }
    } else {
        Ok(req)
    }
}

fn validate_token(
    db: &Arc<Mutex<BTreeMap<String, SessionInfo>>>,
    token: Option<&str>,
    roles: Vec<Role>,
) -> Option<bool> {
    if let (Ok(data), Some(t)) = (db.lock(), token) {
        if let Some(session) = data.get(t) {
            Some(session.is_valid_date() && session.is_valid_role(roles))
        } else {
            None
        }
    } else {
        None
    }
}
