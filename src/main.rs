#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

mod lib;

use lib::utils::{read_dir, fetch_users};
use lib::user::{LoginData, UserData, UserCookie, Role};
use lib::routes::routes;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes())
        .attach(Template::fairing())
        .mount("/", FileServer::from(relative!("/static")))
}