use rocket::{get, launch, post, routes};
use std::path::PathBuf;

#[get("/")]
fn index() -> &'static str {
    "Welcome to API Gateway!"
}

#[get("/health")]
fn health() -> &'static str {
    "OK"
}

#[post("/proxy/<path..>")]
async fn proxy(path: PathBuf) -> Result<String, rocket::http::Status> {
    let target_url = format!("http://httpbin.org/{}", path.display());
    println!("target_url: {}", target_url);
    let client = reqwest::Client::new();

    match client.get(&target_url).send().await {
        Ok(response) => match response.text().await {
            Ok(text) => Ok(text),
            Err(_) => Err(rocket::http::Status::InternalServerError),
        },
        Err(_) => Err(rocket::http::Status::BadGateway),
    }
}

#[launch] // entry point 启动宏
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, health, proxy]) // 注册路由
}
