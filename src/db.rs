use sqlx::{SqlitePool, Result};
use crate::models::Person;

pub async fn create_person(pool: &SqlitePool, person: &Person) -> Result<Person> {
    let result = sqlx::query!(
        "INSERT INTO persons (name, surname, role) VALUES (?, ?, ?)",
        person.name,
        person.surname,
        person.role
    )
    .execute(pool)
    .await?;

    Ok(Person {
        id: Some(result.last_insert_rowid()),
        name: person.name.clone(),
        surname: person.surname.clone(),
        role: person.role.clone(),
    })
}

pub async fn get_person(pool: &SqlitePool, id: i64) -> Result<Option<Person>> {
    let person = sqlx::query_as!(
        Person,
        "SELECT id, name, surname, role FROM persons WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(person)
}

pub async fn get_all_persons(pool: &SqlitePool) -> Result<Vec<Person>> {
    let persons = sqlx::query_as!(
        Person,
        "SELECT id, name, surname, role FROM persons"
    )
    .fetch_all(pool)
    .await?;

    Ok(persons)
}

pub async fn update_person(pool: &SqlitePool, id: i64, person: &Person) -> Result<bool> {
    let result = sqlx::query!(
        "UPDATE persons SET name = ?, surname = ?, role = ? WHERE id = ?",
        person.name,
        person.surname,
        person.role,
        id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_person(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM persons WHERE id = ?",
        id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
