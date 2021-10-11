#![feature(path_try_exists)]
#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};
use rocket::fs::{FileServer, relative, TempFile};
use rocket_dyn_templates::Template;
use rocket::http::{CookieJar, Status, Cookie};
use rocket::form::{Contextual, Errors, Form, FromForm, FromFormField, Result, ValueField, DataField};
use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use evmap::{ReadHandle, WriteHandle};
use rocket_dyn_templates::{context};
use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Cell, RefCell};
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use rocket::data::Data;
use rocket::futures::lock::Mutex;
use rocket::http::ext::IntoCollection;
use rocket::response::Redirect;
use uuid::Uuid;

type FormContext<'v> = rocket::form::Context<'v>;

#[derive(Debug, FromForm)]
struct Account<'v> {
    #[field(validate = len(1..))]
    name: &'v str,
    #[field(validate = len(6..))]
    password: &'v str,
    role: Option<Role>,
}

#[derive(Debug, FromForm)]
struct Submit<'v> {
    account: Account<'v>,
}

#[derive(Debug, FromForm)]
struct UploadSubmit<'v> {
    upload: Upload<'v>,
}

#[derive(Debug, FromForm)]
struct Upload<'v> {
    file: TempFile<'v>,
}


#[derive(Debug, Copy, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(strum_macros::Display)]
enum Role {
    Standard,
    Elevated,
}

impl Role {
    fn from_str(str: &str) -> Result<Role> {
        match str {
            "Standard" => Ok(Role::Standard),
            "Elevated" => Ok(Role::Elevated),
            _ => Err(Errors::default())
        }
    }
}

impl<'v> FromFormField<'v> for Role {
    fn from_value(field: ValueField<'v>) -> Result<'v, Self> {
        match field.value {
            "Standard" => Ok(Role::Standard),
            "Elevated" => Ok(Role::Elevated),
            _ => Err(Errors::default())
        }
    }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct User<'a> {
    name: &'a str,
    password: &'a str,
    role: Role,
    files: Vec<String>,
    id: String,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Session<'a> {
    name: &'a str,
    role: Role,
    files: Vec<String>,
    id: String,
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>, users_r: &State<Arc<Mutex<ReadHandle<String, String>>>>) -> Template {
    let opt = cookies.get("name");
    if let Some(cookie) = opt {
        let username = cookie.value();
        let role = cookies.get("role").unwrap().value();

        let ref db = users_r.lock().await;
        if db.contains_key(username) {
            let db_get = &db.get(username).unwrap();
            let json_str = db_get.get_one().unwrap().as_str();
            let user: User = serde_json::from_str(json_str).unwrap();

            Template::render("index", Session { name: username, role: Role::from_str(role).unwrap(), files: user.files, id: user.id })
        } else {
            Template::render("index", &FormContext::default())
        }
    } else {
        Template::render("index", &FormContext::default())
    }
}

#[get("/login")]
fn login() -> Template {
    Template::render("login", &FormContext::default())
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::named("name"));
    Redirect::to(uri!(login()))
}

#[get("/register")]
fn register() -> Template {
    Template::render("register", &FormContext::default())
}

#[get("/upload")]
fn upload() -> Template {
    Template::render("upload", &FormContext::default())
}

#[post("/login", data = "<form>")]
async fn login_submit<'r>(cookies: &CookieJar<'_>, form: Form<Contextual<'r, Submit<'r>>>, users_r: &State<Arc<Mutex<ReadHandle<String, String>>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            let username = submission.account.name;
            let password = submission.account.password;

            let ref db = users_r.lock().await;
            if db.contains_key(username) {
                let db_get = &db.get(username).unwrap();
                let json_str = db_get.get_one().unwrap().as_str();
                let user: User = serde_json::from_str(json_str).unwrap();
                if user.password == password {
                    println!("submission: {:#?}", submission);
                    cookies.add(Cookie::new("name", String::from(user.name)));
                    Template::render("index", Session { name: username, role: user.role, files: user.files, id: user.id })
                } else {
                    Template::render("login", &form.context)
                }
            } else {
                Template::render("register", &form.context)
            }
        }
        None => {
            println!("No match");
            Template::render("login", &form.context)
        }
    };

    (form.context.status(), template)
}

#[post("/register", data = "<form>")]
async fn register_submit<'r>(form: Form<Contextual<'r, Submit<'r>>>, users_w: &State<Arc<Mutex<WriteHandle<String, String>>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            let user = User {
                name: &submission.account.name.clone(),
                password: submission.account.password.clone(),
                role: submission.account.role.unwrap().clone(),
                files: Vec::new(),
                id: Uuid::new_v4().to_string(),
            };
            let json = serde_json::to_string(&user);
            let mut writer = users_w.lock().await;
            writer.insert(String::from(user.name), json.unwrap());
            writer.refresh();

            fs::create_dir(format!("./files/{}", user.id));
            // println!("submission: {:#?}", &json);
            Template::render("login", &form.context)
        }
        None => {
            println!("No match");
            Template::render("register", &form.context)
        }
    };

    (form.context.status(), template)
}

#[post("/upload", data = "<form>")]
async fn upload_submit<'r>(cookies: &CookieJar<'_>, mut form: Form<Contextual<'r, UploadSubmit<'r>>>, users_r: &State<Arc<Mutex<ReadHandle<String, String>>>>, users_w: &State<Arc<Mutex<WriteHandle<String, String>>>>) -> (Status, Template) {
    let opt = cookies.get("name");
    if let Some(cookie) = opt {
        let username = cookie.value();

        let template = match form.value {
            Some(ref mut submission) => {
                let db = users_r.lock().await;
                let db_get = &db.get(username).unwrap();

                if db.contains_key(username) {
                    let json_str = db_get.get_one().unwrap().as_str();
                    let mut user: User = serde_json::from_str(json_str).unwrap();

                    let filename = submission.upload.file.name().unwrap();
                    let extension = submission.upload.file.content_type().unwrap();
                    let complete_name = format!("{}.{}", filename.clone(), extension.0.sub().as_str());
                    let complete_path = format!("./files/{}/{}", user.id, complete_name.clone());
                    if !fs::try_exists(complete_path.clone()).unwrap() {
                        submission.upload.file.persist_to(complete_path.clone()).await;
                        user.files.push(complete_name);

                        let json = serde_json::to_string(&user);
                        let mut writer = users_w.lock().await;
                        writer.update(String::from(user.name), json.unwrap());
                        writer.refresh();
                    }

                    Template::render("index", Session { name: username, role: user.role, files: user.files, id: user.id })
                } else {
                    Template::render("register", &form.context)
                }
            }
            None => {
                println!("No match");
                Template::render("register", &form.context)
            }
        };
        (form.context.status(), template)
    } else {
        (form.context.status(), Template::render("login", &form.context))
    }
}

#[post("/delete", data = "<form>")]
async fn delete_file<'r>(cookies: &CookieJar<'_>, mut form: Form<Contextual<'r, &'r str>>, users_r: &State<Arc<Mutex<ReadHandle<String, String>>>>, users_w: &State<Arc<Mutex<WriteHandle<String, String>>>>) -> (Status, Template) {
    let opt = cookies.get("name");
    if let Some(cookie) = opt {
        let username = cookie.value();

        let template = match form.value {
            Some(ref mut submission) => {
                let db = users_r.lock().await;
                let db_get = &db.get(username).unwrap();

                if db.contains_key(username) {
                    let json_str = db_get.get_one().unwrap().as_str();
                    let mut user: User = serde_json::from_str(json_str).unwrap();
                    if user.role == Role::Elevated {
                        fs::remove_file(format!("./files/{}/{}", user.id, submission));

                        user.files.remove(user.files.iter().position(|x| x == submission).unwrap());
                        let json = serde_json::to_string(&user);
                        let mut writer = users_w.lock().await;
                        writer.update(String::from(user.name), json.unwrap());
                        writer.refresh();
                    }

                    Template::render("index", Session { name: username, role: user.role, files: user.files, id: user.id })
                } else {
                    Template::render("register", &form.context)
                }
            }
            None => {
                println!("No match");
                Template::render("register", &form.context)
            }
        };
        (form.context.status(), template)
    } else {
        (form.context.status(), Template::render("index", &form.context))
    }
}


#[launch]
fn rocket() -> Rocket<Build> {
    let (users_r, mut users_w) = evmap::new::<String, String>();
    rocket::build()
        .manage(Arc::new(Mutex::new(users_r)))
        .manage(Arc::new(Mutex::new(users_w)))
        .mount("/", routes![index, login, login_submit, register, register_submit, upload, upload_submit, logout, delete_file])
        .attach(Template::fairing())
        .mount("/static", FileServer::from(relative!("/static")))
        .mount("/files", FileServer::from(relative!("/files")).rank(3))
}
