use actix_web::{web, HttpResponse, Responder};
use bcrypt::hash;
use ron::de::from_str;
use uuid::Uuid;

use crate::{core::pretty_config_output, model::error::Error};

use super::{
    io,
    models::{AdminInfo, User},
    schemas::{CreateUserWithAdmin, UserId},
};

pub async fn create_user(body: String, admin: web::Data<AdminInfo>) -> impl Responder {
    let credentials: Result<CreateUserWithAdmin, Error> = match from_str(&body) {
        Ok(x) => Ok(x),
        Err(e) => Err(Error::Ron(e)),
    };

    if let Ok(cred) = credentials {
        if admin.is_valid_hash(&cred.admin_password, &cred.admin_id) {
            let new_user_id = Uuid::new_v4();
            if let Ok(new_user_hash) = hash(&cred.user_info.user_password, admin.cost()) {
                let user = User::new(new_user_id, new_user_hash, cred.user_info.role);
                if let Ok(_) = io::to_users_log(&user) {
                    let user_response = UserId {
                        user_id: new_user_id,
                    };
                    match ron::ser::to_string_pretty(&user_response, pretty_config_output()) {
                        Ok(ron) => HttpResponse::Created().body(ron),
                        Err(_) => HttpResponse::ServiceUnavailable()
                            .body(Error::FailedToCreateUser.to_string()),
                    }
                } else {
                    HttpResponse::ServiceUnavailable().body(Error::FailedToCreateUser.to_string())
                }
            } else {
                HttpResponse::ServiceUnavailable().body(Error::FailedToCreateUser.to_string())
            }
        } else {
            HttpResponse::BadRequest().body(Error::AuthBadRequest.to_string())
        }
    } else {
        HttpResponse::BadRequest().body(credentials.err().unwrap().to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::{auth::io::assert_users_content, http::routes};
    use actix_http::body::ResponseBody;
    use actix_web::{body::Body, test, App};

    #[actix_rt::test]
    async fn create_new_user_ok() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .set_payload("(admin_id: \"your_admin\",admin_password: \"your_password\",user_info: (user_password: \"my_password\",role: [User,],),)")
            .uri("/auth/createUser")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        let body = resp.take_body().as_str().to_string();

        assert!(resp.status().is_success());
        assert!(body.contains("user_id"));
        assert_users_content("roles: [User,],date:");
        assert_users_content("hash: ");
        assert_users_content("id: ");
    }

    #[actix_rt::test]
    async fn create_new_user_wrong_admin() {
        let mut app = test::init_service(App::new().configure(routes)).await;
        let req = test::TestRequest::post()
            .set_payload("(admin_id: \"your_admin\",admin_password: \"nice_password\",user_info: (user_password: \"my_password\",role: [User,],),)")
            .uri("/auth/createUser")
            .to_request();

        let mut resp = test::call_service(&mut app, req).await;
        let body = resp.take_body().as_str().to_string();
        assert!(resp.status().is_client_error());
        assert_eq!(body, "(\n error_type: \"AuthBadRequest\",\n error_message: \"Bad request at authentication endpoint\",\n)");
    }

    trait BodyTest {
        fn as_str(&self) -> &str;
    }

    impl BodyTest for ResponseBody<Body> {
        fn as_str(&self) -> &str {
            match self {
                ResponseBody::Body(ref b) => match b {
                    Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                    _ => panic!(),
                },
                ResponseBody::Other(ref b) => match b {
                    Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                    _ => panic!(),
                },
            }
        }
    }
}
