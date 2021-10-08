/**
 * ! clean up code base
 * ! clean up .tera files (add a base template file, add error template, add "Go back" template file)
 * ! + better error handling
 * ! update login tokens: add e.g. tokens.txt with valid tokens
 * 
 * ? subfolder support for uploading/downloading
 * ? admin role (users can only download, admins can upload/delete)
 * ? different groups (each have their own files they can access)
 */

#[macro_use] extern crate rocket;

use rocket::{
    outcome::IntoOutcome,
    request::{self, FromRequest, Request},
    response::{Redirect},
    http::{Cookie, CookieJar},
    form::{Form, FromForm, Context, },
    fs::{FileServer, TempFile, relative, NamedFile}
};
use std::{
    path::{Path, PathBuf},
    fs,
    str::FromStr
};
use rocket_dyn_templates::Template;
use serde::Serialize;

struct User;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        req.cookies()
            .get_private("is_logged_in")
            .and_then(|cookie| cookie.value().parse().ok())
            .or_forward(())
    }
}

impl FromStr for User {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "true" {
            Ok(User)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, FromForm)]
struct UploadData<'v> {
    #[field(validate = len(1..))]
    name: &'v str,
    file: TempFile<'v>,
}

#[derive(FromForm)]
struct LoginData {
    token: String
}

#[derive(Serialize)]
struct FilePaths {
    paths: Vec<String>
}

#[get("/")]
fn index(_user: User) -> Template {
    Template::render("index", FilePaths { paths: read_dir(&Path::new("files").to_path_buf()).unwrap() })
}

#[get("/", rank = 2)]
fn login_page() -> Template {
    Template::render("login", &Context::default())
}

#[get("/upload")]
fn upload_page(_user: User) -> Template {
    Template::render("upload", &Context::default())
}

#[post("/login", data = "<login_data>")]
fn login(login_data: Form<LoginData>, jar: &CookieJar<'_>) -> Redirect {
    if login_data.token == "secret_token" {
        jar.add_private(Cookie::new("is_logged_in", "true"));
    }

    Redirect::to(uri!(index))
}

#[post("/logout")]
fn logout(_user: User, jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private(Cookie::named("is_logged_in"));
    Redirect::to(uri!(index))
}

#[post("/upload", data = "<upload>")]
async fn upload<'r>(ref mut upload: Form<UploadData<'r>>) -> Template {
    let name = upload.name;

    match upload.file.copy_to(Path::new("files/").join(name)).await {
        Ok(_) => Template::render("success", &Context::default()),
        Err(_) => Template::render("failed", &Context::default()),
    }
}

#[get("/download/<path..>")]
async fn download(_user: User, path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(path).await.ok()
}

#[get("/delete/<path..>")]
fn delete(_user: User, path: PathBuf) -> Template {
    match fs::remove_file(path) {
        Ok(_) => Template::render("deleted", &Context::default()),
        Err(_) => Template::render("failed", &Context::default()),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, login, logout, upload, download, delete, login_page, upload_page])
        .attach(Template::fairing())
        .mount("/", FileServer::from(relative!("/static")))
}

fn read_dir(path: &PathBuf) -> Result<Vec<String>, String> {
    if path.exists() {
        match path.read_dir() {
            Ok(entries) => {
                let mut valid = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(file) = entry.path().to_str() {
                            valid.push(String::from(file));
                        }
                    }
                }
                Ok(valid)
            }
            Err(_) => {
                Err("Couldn't read that directory".to_string())
            }
        }
    } else {
        Err("That path doesn't exist!".to_string())
    }
}