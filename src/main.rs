mod models;
mod db;

use models::Person;
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let pool = SqlitePool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Database connected and migrations applied");

    let new_person = Person::new(
        "John".to_string(),
        "Doe".to_string(),
        "Developer".to_string(),
    );

    let created = db::create_person(&pool, &new_person).await?;
    println!("Created person: {:?}", created);

    let all_persons = db::get_all_persons(&pool).await?;
    println!("All persons: {:?}", all_persons);

    if let Some(id) = created.id {
        if let Some(fetched) = db::get_person(&pool, id).await? {
            println!("Fetched person by id {}: {:?}", id, fetched);
        }

        let updated_person = Person {
            id: Some(id),
            name: "Jane".to_string(),
            surname: "Smith".to_string(),
            role: "Manager".to_string(),
        };

        if db::update_person(&pool, id, &updated_person).await? {
            println!("Updated person with id {}", id);
            if let Some(updated) = db::get_person(&pool, id).await? {
                println!("After update: {:?}", updated);
            }
        }
    }

    Ok(())
}
