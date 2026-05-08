use rocket::{State, serde::json::Json, http::Status};
use rocket::{get, post, put, delete, routes};
use rocket_dyn_templates::{Template, context};
use serde::Deserialize;
use sqlx::SqlitePool;
use crate::models::{Person, PersonWithRoles, Role};
use crate::db;

#[get("/people")]
pub async fn people_page(pool: &State<SqlitePool>) -> Result<Template, Status> {
    let persons = db::get_all_persons_with_roles(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render("people", context! {
        persons: persons
    }))
}

#[get("/people/<id>")]
pub async fn edit_person_page(pool: &State<SqlitePool>, id: i64) -> Result<Template, Status> {
    let person_with_roles = db::get_person_with_roles(pool, id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let all_roles = db::get_all_roles(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render("edit_person", context! {
        person: person_with_roles.person,
        person_roles: person_with_roles.roles,
        all_roles: all_roles
    }))
}

#[derive(Deserialize)]
pub struct PersonWithRolesInput {
    #[serde(flatten)]
    person: Person,
    roles: Vec<String>,
}

#[get("/persons")]
pub async fn get_all_persons(pool: &State<SqlitePool>) -> Result<Json<Vec<PersonWithRoles>>, Status> {
    db::get_all_persons_with_roles(pool)
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[get("/persons/<id>")]
pub async fn get_person(pool: &State<SqlitePool>, id: i64) -> Result<Json<PersonWithRoles>, Status> {
    db::get_person_with_roles(pool, id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .map(Json)
        .ok_or(Status::NotFound)
}

#[post("/persons", data = "<input>")]
pub async fn create_person(
    pool: &State<SqlitePool>,
    input: Json<PersonWithRolesInput>,
) -> Result<Json<PersonWithRoles>, Status> {
    let input = input.into_inner();
    let created_person = db::create_person(pool, &input.person)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let person_id = created_person.id.ok_or(Status::InternalServerError)?;

    db::set_person_roles(pool, person_id, &input.roles)
        .await
        .map_err(|_| Status::InternalServerError)?;

    db::get_person_with_roles(pool, person_id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .map(Json)
        .ok_or(Status::InternalServerError)
}

#[put("/persons/<id>", data = "<input>")]
pub async fn update_person(
    pool: &State<SqlitePool>,
    id: i64,
    input: Json<PersonWithRolesInput>,
) -> Result<Status, Status> {
    let input = input.into_inner();

    let updated = db::update_person(pool, id, &input.person)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !updated {
        return Err(Status::NotFound);
    }

    db::set_person_roles(pool, id, &input.roles)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Status::Ok)
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
                    surname TEXT NOT NULL
                )"
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS roles (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE
                )"
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS person_roles (
                    person_id INTEGER NOT NULL,
                    role_id INTEGER NOT NULL,
                    PRIMARY KEY (person_id, role_id),
                    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
                    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
                )"
            )
            .execute(&pool)
            .await
            .unwrap();

            pool
        });

        let rocket = rocket::build()
            .manage(pool)
            .attach(Template::fairing())
            .mount("/", routes![
                people_page,
                edit_person_page,
            ])
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
            .body(r#"{"name":"John","surname":"Doe","roles":["Developer"]}"#)
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
            .body(r#"{"name":"Jane","surname":"Smith","roles":["Manager"]}"#)
            .dispatch();

        let response = client
            .put("/api/persons/1")
            .header(ContentType::JSON)
            .body(r#"{"name":"Jane","surname":"Smith","roles":["Senior Manager"]}"#)
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
            .body(r#"{"name":"Test","surname":"User","roles":["Role"]}"#)
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_delete_person() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Delete","surname":"Me","roles":["Test"]}"#)
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
            .body(r#"{"name":"Alice","surname":"Brown","roles":["Designer"]}"#)
            .dispatch();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Bob","surname":"Green","roles":["Developer"]}"#)
            .dispatch();

        let response = client.get("/api/persons").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Alice"));
        assert!(body.contains("Bob"));
    }

    #[test]
    fn test_people_page_empty() {
        let client = setup_test_rocket();

        let response = client.get("/people").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains("<title>People</title>"));
        assert!(body.contains("<th>First Name</th>"));
        assert!(body.contains("<th>Last Name</th>"));
        assert!(body.contains("<th>Roles</th>"));
    }

    #[test]
    fn test_people_page_with_data() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"John","surname":"Doe","roles":["Developer"]}"#)
            .dispatch();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Jane","surname":"Smith","roles":["Manager"]}"#)
            .dispatch();

        let response = client.get("/people").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains("<title>People</title>"));
        assert!(body.contains("<td>John</td>"));
        assert!(body.contains("<td>Doe</td>"));
        assert!(body.contains("<td>Developer</td>"));
        assert!(body.contains("<td>Jane</td>"));
        assert!(body.contains("<td>Smith</td>"));
        assert!(body.contains("<td>Manager</td>"));
    }

    #[test]
    fn test_people_page_includes_htmx() {
        let client = setup_test_rocket();

        let response = client.get("/people").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains("htmx.org"));
    }

    #[test]
    fn test_edit_person_page() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"John","surname":"Doe","roles":["Developer"]}"#)
            .dispatch();

        let response = client.get("/people/1").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains("<title>Edit Person</title>"));
        assert!(body.contains(r#"value="John""#));
        assert!(body.contains(r#"value="Doe""#));
        assert!(body.contains(r#"value="Developer""#));
        assert!(body.contains("Save"));
        assert!(body.contains("Cancel"));
        assert!(body.contains("Delete"));
    }

    #[test]
    fn test_edit_person_page_not_found() {
        let client = setup_test_rocket();

        let response = client.get("/people/999").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_edit_person_page_has_form() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Alice","surname":"Smith","roles":["Manager"]}"#)
            .dispatch();

        let response = client.get("/people/1").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains(r#"<form id="editForm""#));
        assert!(body.contains(r#"<input type="text" id="name""#));
        assert!(body.contains(r#"<input type="text" id="surname""#));
        assert!(body.contains(r#"<input type="text" id="roles""#));
    }

    #[test]
    fn test_edit_page_has_delete_button() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test","surname":"User","roles":["Temp"]}"#)
            .dispatch();

        let response = client.get("/people/1").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains(r#"hx-delete="/api/persons/1""#));
        assert!(body.contains(r#"hx-confirm="Are you sure you want to delete this person?""#));
    }

    #[test]
    fn test_people_page_rows_clickable() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Bob","surname":"Brown","roles":["Tester"]}"#)
            .dispatch();

        let response = client.get("/people").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains(r#"onclick="window.location='/people/1'""#));
        assert!(body.contains("cursor: pointer"));
    }
}
