use rocket::{State, serde::json::Json, http::Status};
use rocket::{get, post, put, delete, routes};
use rocket_dyn_templates::{Template, context};
use sqlx::SqlitePool;
use crate::models::Person;
use crate::db;

#[get("/people")]
pub async fn people_page(pool: &State<SqlitePool>) -> Result<Template, Status> {
    let persons = db::get_all_persons(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render("people", context! {
        persons: persons
    }))
}

#[get("/persons")]
pub async fn get_all_persons(pool: &State<SqlitePool>) -> Result<Json<Vec<Person>>, Status> {
    db::get_all_persons(pool)
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[get("/persons/<id>")]
pub async fn get_person(pool: &State<SqlitePool>, id: i64) -> Result<Json<Person>, Status> {
    db::get_person(pool, id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .map(Json)
        .ok_or(Status::NotFound)
}

#[post("/persons", data = "<person>")]
pub async fn create_person(
    pool: &State<SqlitePool>,
    person: Json<Person>,
) -> Result<Json<Person>, Status> {
    db::create_person(pool, &person.into_inner())
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[put("/persons/<id>", data = "<person>")]
pub async fn update_person(
    pool: &State<SqlitePool>,
    id: i64,
    person: Json<Person>,
) -> Result<Status, Status> {
    db::update_person(pool, id, &person.into_inner())
        .await
        .map_err(|_| Status::InternalServerError)?
        .then_some(Status::Ok)
        .ok_or(Status::NotFound)
}

#[delete("/persons/<id>")]
pub async fn delete_person(pool: &State<SqlitePool>, id: i64) -> Result<Status, Status> {
    db::delete_person(pool, id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .then_some(Status::NoContent)
        .ok_or(Status::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::http::{ContentType, Status};

    fn setup_test_rocket() -> Client {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool = rt.block_on(async {
            let pool = SqlitePool::connect(":memory:").await.unwrap();

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS persons (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    surname TEXT NOT NULL,
                    role TEXT NOT NULL
                )"
            )
            .execute(&pool)
            .await
            .unwrap();

            pool
        });

        let rocket = rocket::build()
            .manage(pool)
            .mount("/api", routes![
                get_all_persons,
                get_person,
                create_person,
                update_person,
                delete_person,
            ]);

        Client::tracked(rocket).expect("valid rocket instance")
    }

    #[test]
    fn test_get_all_persons_empty() {
        let client = setup_test_rocket();
        let response = client.get("/api/persons").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "[]");
    }

    #[test]
    fn test_create_and_get_person() {
        let client = setup_test_rocket();

        let response = client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"John","surname":"Doe","role":"Developer"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("John"));
        assert!(body.contains("Doe"));
        assert!(body.contains("Developer"));

        let response = client.get("/api/persons/1").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("John"));
    }

    #[test]
    fn test_get_person_not_found() {
        let client = setup_test_rocket();

        let response = client.get("/api/persons/999").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_update_person() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Jane","surname":"Smith","role":"Manager"}"#)
            .dispatch();

        let response = client
            .put("/api/persons/1")
            .header(ContentType::JSON)
            .body(r#"{"name":"Jane","surname":"Smith","role":"Senior Manager"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let response = client.get("/api/persons/1").dispatch();
        let body = response.into_string().unwrap();
        assert!(body.contains("Senior Manager"));
    }

    #[test]
    fn test_update_person_not_found() {
        let client = setup_test_rocket();

        let response = client
            .put("/api/persons/999")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test","surname":"User","role":"Role"}"#)
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_delete_person() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Delete","surname":"Me","role":"Test"}"#)
            .dispatch();

        let response = client.delete("/api/persons/1").dispatch();
        assert_eq!(response.status(), Status::NoContent);

        let response = client.get("/api/persons/1").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_delete_person_not_found() {
        let client = setup_test_rocket();

        let response = client.delete("/api/persons/999").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_get_all_persons_multiple() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Alice","surname":"Brown","role":"Designer"}"#)
            .dispatch();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Bob","surname":"Green","role":"Developer"}"#)
            .dispatch();

        let response = client.get("/api/persons").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Alice"));
        assert!(body.contains("Bob"));
    }
}
