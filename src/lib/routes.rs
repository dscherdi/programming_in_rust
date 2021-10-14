use rocket::{
    Route,
    response::{Redirect},
    http::{Cookie, CookieJar},
    form::{Form, FromForm, Context, },
    fs::{TempFile, NamedFile}
};
use std::{
    path::{Path, PathBuf},
    fs
};
use rocket_dyn_templates::Template;
use serde::{Serialize};

use crate::{UserCookie, Role, LoginData, read_dir, fetch_users};

#[derive(Debug, FromForm)]
struct UploadData<'v> {
    #[field(validate = len(1..))]
    name: &'v str,
    file: TempFile<'v>,
}

#[derive(Serialize)]
struct FilePaths {
    paths: Vec<String>
}

#[derive(Serialize)]
struct Message {
    header: &'static str,
    content: &'static str,
}

#[get("/")]
fn index(user: UserCookie) -> Template {
    let mut paths: Vec<PathBuf> = Vec::new();
    if user.role != Role::Elevated {
        paths.push(Path::new("files").join(user.group));
    } else {
        match Path::new("files").read_dir() {
            Ok(files) => {
                for file in files {
                    if let Ok(file) = file {
                        paths.push(
                            Path::new("files")
                                .join(
                                    file.path()
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                )
                            );
                    }
                }
            },
            Err(_) => {},
        };
    }

    Template::render("index", FilePaths {
        paths: read_dir(&paths)
    })
}

#[get("/", rank = 2)]
fn login_page() -> Template {
    Template::render("login", &Context::default())
}

#[get("/upload")]
fn upload_page(_user: UserCookie) -> Template {
    Template::render("upload", &Context::default())
}

#[post("/login", data = "<login_data>")]
fn login(login_data: Form<LoginData>, jar: &CookieJar<'_>) -> Redirect {
	let user = LoginData {
		username: login_data.username.to_string(),
		password: login_data.password.to_string()
	};

	for u in fetch_users() {
		if user == u {
			jar.add_private(
                Cookie::new(
                    "user",
                    serde_json::to_string(&UserCookie { group: u.group, role: u.role }).unwrap()
                )
            );
			break;
		}
	}

    Redirect::to(uri!(index))
}

#[post("/logout")]
fn logout(_user: UserCookie, jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private(Cookie::named("user"));
    Redirect::to(uri!(index))
}

#[post("/upload", data = "<upload>")]
async fn upload<'r>(user: UserCookie, ref mut upload: Form<UploadData<'r>>) -> Template {
	let filename = upload.name;

    let path = Path::new("files").join(user.group);
    match fs::create_dir_all(&path) {
        Ok(_) => {},
        Err(_) => return Template::render(
            "message",
            &Message { header: "Error!", content: "Something went wrong uploading that file." }
        ),
    };

    match upload.file.copy_to(
        path.join(filename)
    ).await {
        Ok(_) => return Template::render(
            "message",
            &Message { header: "Success!", content: "File was uploaded." }
        ),
        Err(_) => return Template::render(
            "message",
            &Message { header: "Error!", content: "Something went wrong uploading that file." }
        ),
    }
}

#[get("/download/<group>/<path>")]
async fn download(user: UserCookie, group: &str, path: &str) -> Option<NamedFile> {
    if group != user.group && user.role != Role::Elevated {
        println!("{:?}", user);
        return None
    }

    NamedFile::open(Path::new("files").join(group).join(path)).await.ok()
}

#[get("/delete/<group>/<path>")]
fn delete(user: UserCookie, group: &str, path: &str) -> Template {
    if group != user.group && user.role != Role::Elevated {
        return Template::render(
            "message",
            &Message { header: "Missing access!", content: "You don't have access to this file." }
        );
    }

    match fs::remove_file(Path::new("files").join(group).join(path)) {
        Ok(_) => Template::render(
            "message",
            &Message { header: "Success!", content: "File was deleted." }
        ),
        Err(_) => Template::render(
            "message",
            &Message { header: "Error!", content: "Something went wrong deleting that file." }
        ),
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        index, login, logout, upload,
        download, delete, login_page, upload_page
    ]
}