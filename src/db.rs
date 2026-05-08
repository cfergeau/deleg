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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Person;

    async fn setup_test_db() -> SqlitePool {
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
    }

    #[tokio::test]
    async fn test_create_person() {
        let pool = setup_test_db().await;

        let person = Person::new(
            "John".to_string(),
            "Doe".to_string(),
            "Developer".to_string(),
        );

        let created = create_person(&pool, &person).await.unwrap();

        assert!(created.id.is_some());
        assert_eq!(created.name, "John");
        assert_eq!(created.surname, "Doe");
        assert_eq!(created.role, "Developer");
    }

    #[tokio::test]
    async fn test_get_person() {
        let pool = setup_test_db().await;

        let person = Person::new(
            "Jane".to_string(),
            "Smith".to_string(),
            "Manager".to_string(),
        );

        let created = create_person(&pool, &person).await.unwrap();
        let id = created.id.unwrap();

        let fetched = get_person(&pool, id).await.unwrap();

        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, Some(id));
        assert_eq!(fetched.name, "Jane");
        assert_eq!(fetched.surname, "Smith");
        assert_eq!(fetched.role, "Manager");
    }

    #[tokio::test]
    async fn test_get_person_not_found() {
        let pool = setup_test_db().await;

        let result = get_person(&pool, 999).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_all_persons() {
        let pool = setup_test_db().await;

        let person1 = Person::new("Alice".to_string(), "Brown".to_string(), "Designer".to_string());
        let person2 = Person::new("Bob".to_string(), "Green".to_string(), "Developer".to_string());

        create_person(&pool, &person1).await.unwrap();
        create_person(&pool, &person2).await.unwrap();

        let all = get_all_persons(&pool).await.unwrap();

        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "Alice");
        assert_eq!(all[1].name, "Bob");
    }

    #[tokio::test]
    async fn test_update_person() {
        let pool = setup_test_db().await;

        let person = Person::new("Old".to_string(), "Name".to_string(), "Role".to_string());
        let created = create_person(&pool, &person).await.unwrap();
        let id = created.id.unwrap();

        let updated_person = Person {
            id: Some(id),
            name: "New".to_string(),
            surname: "Updated".to_string(),
            role: "NewRole".to_string(),
        };

        let result = update_person(&pool, id, &updated_person).await.unwrap();
        assert!(result);

        let fetched = get_person(&pool, id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "New");
        assert_eq!(fetched.surname, "Updated");
        assert_eq!(fetched.role, "NewRole");
    }

    #[tokio::test]
    async fn test_update_person_not_found() {
        let pool = setup_test_db().await;

        let person = Person::new("Test".to_string(), "User".to_string(), "Role".to_string());
        let result = update_person(&pool, 999, &person).await.unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_delete_person() {
        let pool = setup_test_db().await;

        let person = Person::new("Delete".to_string(), "Me".to_string(), "Test".to_string());
        let created = create_person(&pool, &person).await.unwrap();
        let id = created.id.unwrap();

        let result = delete_person(&pool, id).await.unwrap();
        assert!(result);

        let fetched = get_person(&pool, id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_person_not_found() {
        let pool = setup_test_db().await;

        let result = delete_person(&pool, 999).await.unwrap();

        assert!(!result);
    }
}
