use log::debug;
use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

use math_battle::UserId;

pub fn set_user_id(cookies: Cookies) -> UserId {
    let user_id = match cookies.get("user_id") {
        Some(cookie) => {
            let user_id = cookie.value().to_string();
            debug!("Took an existing user id {user_id} from cookies");
            user_id
        }
        None => {
            let user_id = UserId::random();

            cookies.add(
                Cookie::build(("user_id", user_id.value().to_string()))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::None)
                    .max_age(Duration::hours(1))
                    .build(),
            );

            debug!("Set new user id {user_id:?} to cookies");

            user_id.value().to_string()
        }
    };

    UserId::try_from(user_id.as_str()).expect("failed to parse user id")
}
