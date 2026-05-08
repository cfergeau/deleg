use rocket::{State, serde::json::Json, http::Status};
use rocket::{get, post, put, delete};
use sqlx::SqlitePool;
use crate::models::Person;
use crate::db;

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
