mod models;
mod db;
mod routes;

use rocket::{launch, routes};
use rocket_dyn_templates::Template;
use sqlx::sqlite::SqlitePool;

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    rocket::build()
        .manage(pool)
        .attach(Template::fairing())
        .mount("/", routes![
            routes::people_page,
        ])
        .mount("/api", routes![
            routes::get_all_persons,
            routes::get_person,
            routes::create_person,
            routes::update_person,
            routes::delete_person,
        ])
}
