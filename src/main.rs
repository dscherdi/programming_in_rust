#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

mod lib;
use lib::utils::{read_dir, fetch_users, Data};
use lib::user::{LoginData, UserData, UserCookie, Role};
use lib::routes::routes;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes())
        .mount("/", FileServer::from(relative!("/static")))
        .attach(Template::fairing())
        .manage(fetch_users())
}