use rocket::{get, launch, routes};

#[get("/")]
fn index() -> &'static str {
    "Welcome to API Gateway!"
}

#[get("/health")]
fn health() -> &'static str {
    "OK"
}

#[launch] // entry point 启动宏
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, health]) // 注册路由
}
