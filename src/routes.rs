use rocket::{State, serde::json::Json, http::Status};
use rocket::{get, post, put, delete, routes};
use rocket_dyn_templates::{Template, context};
use serde::Deserialize;
use sqlx::SqlitePool;
use crate::models::{Person, PersonWithRoles, Role, RoleAssignment};
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

#[get("/roles")]
pub async fn roles_page(pool: &State<SqlitePool>) -> Result<Template, Status> {
    let roles = db::get_all_roles(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render("roles", context! {
        roles: roles
    }))
}

#[derive(Deserialize)]
pub struct PersonWithRolesInput {
    #[serde(flatten)]
    person: Person,
    roles: Vec<RoleAssignment>,
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

// Role API endpoints
#[get("/roles")]
pub async fn get_all_roles(pool: &State<SqlitePool>) -> Result<Json<Vec<Role>>, Status> {
    db::get_all_roles(pool)
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[get("/roles/<id>")]
pub async fn get_role(pool: &State<SqlitePool>, id: i64) -> Result<Json<Role>, Status> {
    db::get_role(pool, id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .map(Json)
        .ok_or(Status::NotFound)
}

#[post("/roles", data = "<role>")]
pub async fn create_role(
    pool: &State<SqlitePool>,
    role: Json<Role>,
) -> Result<Json<Role>, Status> {
    db::create_role(pool, &role.into_inner())
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[put("/roles/<id>", data = "<role>")]
pub async fn update_role(
    pool: &State<SqlitePool>,
    id: i64,
    role: Json<Role>,
) -> Result<Json<Role>, Status> {
    let role_data = role.into_inner();

    let updated = db::update_role(pool, id, &role_data)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !updated {
        return Err(Status::NotFound);
    }

    // Return the updated role
    db::get_role(pool, id)
        .await
        .map_err(|_| Status::InternalServerError)?
        .map(Json)
        .ok_or(Status::InternalServerError)
}

#[delete("/roles/<id>")]
pub async fn delete_role(pool: &State<SqlitePool>, id: i64) -> Result<Status, Status> {
    db::delete_role(pool, id)
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
                    name TEXT NOT NULL UNIQUE,
                    delegation_hours REAL NOT NULL DEFAULT 0.0
                )"
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS person_roles (
                    person_id INTEGER NOT NULL,
                    role_id INTEGER NOT NULL,
                    startdate TEXT,
                    enddate TEXT,
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
                roles_page,
            ])
            .mount("/api", routes![
                get_all_persons,
                get_person,
                create_person,
                update_person,
                delete_person,
                get_all_roles,
                get_role,
                create_role,
                update_role,
                delete_role,
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
            .body(r#"{"name":"John","surname":"Doe","roles":[{"role_name":"Developer","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"Jane","surname":"Smith","roles":[{"role_name":"Manager","startdate":null,"enddate":null}]}"#)
            .dispatch();

        let response = client
            .put("/api/persons/1")
            .header(ContentType::JSON)
            .body(r#"{"name":"Jane","surname":"Smith","roles":[{"role_name":"Senior Manager","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"Test","surname":"User","roles":[{"role_name":"Role","startdate":null,"enddate":null}]}"#)
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_delete_person() {
        let client = setup_test_rocket();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Delete","surname":"Me","roles":[{"role_name":"Test","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"Alice","surname":"Brown","roles":[{"role_name":"Designer","startdate":null,"enddate":null}]}"#)
            .dispatch();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Bob","surname":"Green","roles":[{"role_name":"Developer","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"John","surname":"Doe","roles":[{"role_name":"Developer","startdate":null,"enddate":null}]}"#)
            .dispatch();

        client
            .post("/api/persons")
            .header(ContentType::JSON)
            .body(r#"{"name":"Jane","surname":"Smith","roles":[{"role_name":"Manager","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"John","surname":"Doe","roles":[{"role_name":"Developer","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"Alice","surname":"Smith","roles":[{"role_name":"Manager","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"Test","surname":"User","roles":[{"role_name":"Temp","startdate":null,"enddate":null}]}"#)
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
            .body(r#"{"name":"Bob","surname":"Brown","roles":[{"role_name":"Tester","startdate":null,"enddate":null}]}"#)
            .dispatch();

        let response = client.get("/people").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains(r#"onclick="window.location='/people/1'""#));
        assert!(body.contains("cursor: pointer"));
    }

    // Role API tests
    #[test]
    fn test_get_all_roles_empty() {
        let client = setup_test_rocket();
        let response = client.get("/api/roles").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "[]");
    }

    #[test]
    fn test_create_and_get_role() {
        let client = setup_test_rocket();

        let response = client
            .post("/api/roles")
            .header(ContentType::JSON)
            .body(r#"{"name":"Élu titulaire CSE","delegation_hours":18.0}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Élu titulaire CSE"));
        assert!(body.contains("18"));

        let response = client.get("/api/roles/1").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Élu titulaire CSE"));
        assert!(body.contains("18"));
    }

    #[test]
    fn test_get_role_not_found() {
        let client = setup_test_rocket();

        let response = client.get("/api/roles/999").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_update_role() {
        let client = setup_test_rocket();

        client
            .post("/api/roles")
            .header(ContentType::JSON)
            .body(r#"{"name":"Délégué syndical","delegation_hours":12.0}"#)
            .dispatch();

        let response = client
            .put("/api/roles/1")
            .header(ContentType::JSON)
            .body(r#"{"name":"Délégué syndical","delegation_hours":24.0}"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let response = client.get("/api/roles/1").dispatch();
        let body = response.into_string().unwrap();
        assert!(body.contains("24"));
    }

    #[test]
    fn test_update_role_not_found() {
        let client = setup_test_rocket();

        let response = client
            .put("/api/roles/999")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test Role","delegation_hours":10.0}"#)
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_delete_role() {
        let client = setup_test_rocket();

        client
            .post("/api/roles")
            .header(ContentType::JSON)
            .body(r#"{"name":"Temporary Role","delegation_hours":5.0}"#)
            .dispatch();

        let response = client.delete("/api/roles/1").dispatch();
        assert_eq!(response.status(), Status::NoContent);

        let response = client.get("/api/roles/1").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_delete_role_not_found() {
        let client = setup_test_rocket();

        let response = client.delete("/api/roles/999").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_get_all_roles_multiple() {
        let client = setup_test_rocket();

        client
            .post("/api/roles")
            .header(ContentType::JSON)
            .body(r#"{"name":"Élu titulaire CSE","delegation_hours":18.0}"#)
            .dispatch();

        client
            .post("/api/roles")
            .header(ContentType::JSON)
            .body(r#"{"name":"Délégué syndical","delegation_hours":24.0}"#)
            .dispatch();

        let response = client.get("/api/roles").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Élu titulaire CSE"));
        assert!(body.contains("Délégué syndical"));
        assert!(body.contains("18"));
        assert!(body.contains("24"));
    }

    #[test]
    fn test_roles_page_empty() {
        let client = setup_test_rocket();

        let response = client.get("/roles").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains("<title>Roles</title>"));
        assert!(body.contains("Role Name"));
        assert!(body.contains("Delegation Hours"));
    }

    #[test]
    fn test_roles_page_with_data() {
        let client = setup_test_rocket();

        client
            .post("/api/roles")
            .header(ContentType::JSON)
            .body(r#"{"name":"Élu titulaire CSE","delegation_hours":18.0}"#)
            .dispatch();

        client
            .post("/api/roles")
            .header(ContentType::JSON)
            .body(r#"{"name":"Délégué syndical","delegation_hours":24.0}"#)
            .dispatch();

        let response = client.get("/roles").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert!(body.contains("<title>Roles</title>"));
        assert!(body.contains("Élu titulaire CSE"));
        assert!(body.contains("Délégué syndical"));
        assert!(body.contains("18"));
        assert!(body.contains("24"));
    }
}
