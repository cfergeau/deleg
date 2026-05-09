use sqlx::{SqlitePool, Result};
use crate::models::{Person, PersonWithRoles, Role};

// Person CRUD operations
pub async fn create_person(pool: &SqlitePool, person: &Person) -> Result<Person> {
    let result = sqlx::query!(
        "INSERT INTO persons (name, surname) VALUES (?, ?)",
        person.name,
        person.surname
    )
    .execute(pool)
    .await?;

    Ok(Person {
        id: Some(result.last_insert_rowid()),
        name: person.name.clone(),
        surname: person.surname.clone(),
    })
}

pub async fn get_person(pool: &SqlitePool, id: i64) -> Result<Option<Person>> {
    let person = sqlx::query_as!(
        Person,
        "SELECT id, name, surname FROM persons WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(person)
}

pub async fn get_person_with_roles(pool: &SqlitePool, id: i64) -> Result<Option<PersonWithRoles>> {
    let person = get_person(pool, id).await?;

    match person {
        Some(p) => {
            let roles = get_person_roles(pool, id).await?;
            Ok(Some(PersonWithRoles {
                person: p,
                roles,
            }))
        }
        None => Ok(None),
    }
}

pub async fn get_all_persons(pool: &SqlitePool) -> Result<Vec<Person>> {
    let persons = sqlx::query_as!(
        Person,
        "SELECT id, name, surname FROM persons"
    )
    .fetch_all(pool)
    .await?;

    Ok(persons)
}

pub async fn get_all_persons_with_roles(pool: &SqlitePool) -> Result<Vec<PersonWithRoles>> {
    let persons = get_all_persons(pool).await?;
    let mut persons_with_roles = Vec::new();

    for person in persons {
        let person_id = person.id.unwrap();
        let roles = get_person_roles(pool, person_id).await?;
        persons_with_roles.push(PersonWithRoles {
            person,
            roles,
        });
    }

    Ok(persons_with_roles)
}

pub async fn update_person(pool: &SqlitePool, id: i64, person: &Person) -> Result<bool> {
    let result = sqlx::query!(
        "UPDATE persons SET name = ?, surname = ? WHERE id = ?",
        person.name,
        person.surname,
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

// Role CRUD operations
pub async fn create_role(pool: &SqlitePool, role: &Role) -> Result<Role> {
    let result = sqlx::query!(
        "INSERT INTO roles (name, delegation_hours) VALUES (?, ?)",
        role.name,
        role.delegation_hours
    )
    .execute(pool)
    .await?;

    Ok(Role {
        id: Some(result.last_insert_rowid()),
        name: role.name.clone(),
        delegation_hours: role.delegation_hours,
    })
}

pub async fn get_role(pool: &SqlitePool, id: i64) -> Result<Option<Role>> {
    let role = sqlx::query_as!(
        Role,
        "SELECT id, name, delegation_hours FROM roles WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(role)
}

pub async fn get_role_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Role>> {
    let role = sqlx::query_as!(
        Role,
        "SELECT id, name, delegation_hours FROM roles WHERE name = ?",
        name
    )
    .fetch_optional(pool)
    .await?;

    Ok(role)
}

pub async fn get_all_roles(pool: &SqlitePool) -> Result<Vec<Role>> {
    let roles = sqlx::query_as!(
        Role,
        "SELECT id, name, delegation_hours FROM roles"
    )
    .fetch_all(pool)
    .await?;

    Ok(roles)
}

pub async fn update_role(pool: &SqlitePool, id: i64, role: &Role) -> Result<bool> {
    let result = sqlx::query!(
        "UPDATE roles SET name = ?, delegation_hours = ? WHERE id = ?",
        role.name,
        role.delegation_hours,
        id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_role(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM roles WHERE id = ?",
        id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Person-Role relationship operations
pub async fn add_role_to_person(pool: &SqlitePool, person_id: i64, role_id: i64) -> Result<()> {
    sqlx::query!(
        "INSERT OR IGNORE INTO person_roles (person_id, role_id) VALUES (?, ?)",
        person_id,
        role_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_role_from_person(pool: &SqlitePool, person_id: i64, role_id: i64) -> Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM person_roles WHERE person_id = ? AND role_id = ?",
        person_id,
        role_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_person_roles(pool: &SqlitePool, person_id: i64) -> Result<Vec<String>> {
    let roles = sqlx::query!(
        "SELECT r.name FROM roles r
         JOIN person_roles pr ON r.id = pr.role_id
         WHERE pr.person_id = ?
         ORDER BY r.name",
        person_id
    )
    .fetch_all(pool)
    .await?;

    Ok(roles.into_iter().map(|r| r.name).collect())
}

pub async fn set_person_roles(pool: &SqlitePool, person_id: i64, role_names: &[String]) -> Result<()> {
    // Remove all existing roles
    sqlx::query!(
        "DELETE FROM person_roles WHERE person_id = ?",
        person_id
    )
    .execute(pool)
    .await?;

    // Add new roles
    for role_name in role_names {
        // Get or create role
        let role = match get_role_by_name(pool, role_name).await? {
            Some(r) => r,
            None => create_role(pool, &Role::new(role_name.clone(), 0.0)).await?,
        };

        if let Some(role_id) = role.id {
            add_role_to_person(pool, person_id, role_id).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqlitePool {
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
                PRIMARY KEY (person_id, role_id),
                FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
                FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
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

        let person = Person::new("John".to_string(), "Doe".to_string());

        let created = create_person(&pool, &person).await.unwrap();

        assert!(created.id.is_some());
        assert_eq!(created.name, "John");
        assert_eq!(created.surname, "Doe");
    }

    #[tokio::test]
    async fn test_get_person() {
        let pool = setup_test_db().await;

        let person = Person::new("Jane".to_string(), "Smith".to_string());
        let created = create_person(&pool, &person).await.unwrap();
        let id = created.id.unwrap();

        let fetched = get_person(&pool, id).await.unwrap();

        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, Some(id));
        assert_eq!(fetched.name, "Jane");
        assert_eq!(fetched.surname, "Smith");
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

        let person1 = Person::new("Alice".to_string(), "Brown".to_string());
        let person2 = Person::new("Bob".to_string(), "Green".to_string());

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

        let person = Person::new("Old".to_string(), "Name".to_string());
        let created = create_person(&pool, &person).await.unwrap();
        let id = created.id.unwrap();

        let updated_person = Person {
            id: Some(id),
            name: "New".to_string(),
            surname: "Updated".to_string(),
        };

        let result = update_person(&pool, id, &updated_person).await.unwrap();
        assert!(result);

        let fetched = get_person(&pool, id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "New");
        assert_eq!(fetched.surname, "Updated");
    }

    #[tokio::test]
    async fn test_update_person_not_found() {
        let pool = setup_test_db().await;

        let person = Person::new("Test".to_string(), "User".to_string());
        let result = update_person(&pool, 999, &person).await.unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_delete_person() {
        let pool = setup_test_db().await;

        let person = Person::new("Delete".to_string(), "Me".to_string());
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

    #[tokio::test]
    async fn test_create_role() {
        let pool = setup_test_db().await;

        let role = Role::new("Developer".to_string(), 18.0);
        let created = create_role(&pool, &role).await.unwrap();

        assert!(created.id.is_some());
        assert_eq!(created.name, "Developer");
        assert_eq!(created.delegation_hours, 18.0);
    }

    #[tokio::test]
    async fn test_get_role() {
        let pool = setup_test_db().await;

        let role = Role::new("Manager".to_string(), 20.0);
        let created = create_role(&pool, &role).await.unwrap();
        let id = created.id.unwrap();

        let fetched = get_role(&pool, id).await.unwrap();

        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, Some(id));
        assert_eq!(fetched.name, "Manager");
        assert_eq!(fetched.delegation_hours, 20.0);
    }

    #[tokio::test]
    async fn test_get_role_by_name() {
        let pool = setup_test_db().await;

        let role = Role::new("Designer".to_string(), 15.0);
        create_role(&pool, &role).await.unwrap();

        let fetched = get_role_by_name(&pool, "Designer").await.unwrap();

        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.name, "Designer");
        assert_eq!(fetched.delegation_hours, 15.0);
    }

    #[tokio::test]
    async fn test_add_role_to_person() {
        let pool = setup_test_db().await;

        let person = Person::new("John".to_string(), "Doe".to_string());
        let created_person = create_person(&pool, &person).await.unwrap();
        let person_id = created_person.id.unwrap();

        let role = Role::new("Developer".to_string(), 18.0);
        let created_role = create_role(&pool, &role).await.unwrap();
        let role_id = created_role.id.unwrap();

        add_role_to_person(&pool, person_id, role_id).await.unwrap();

        let roles = get_person_roles(&pool, person_id).await.unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0], "Developer");
    }

    #[tokio::test]
    async fn test_remove_role_from_person() {
        let pool = setup_test_db().await;

        let person = Person::new("Jane".to_string(), "Smith".to_string());
        let created_person = create_person(&pool, &person).await.unwrap();
        let person_id = created_person.id.unwrap();

        let role = Role::new("Manager".to_string(), 20.0);
        let created_role = create_role(&pool, &role).await.unwrap();
        let role_id = created_role.id.unwrap();

        add_role_to_person(&pool, person_id, role_id).await.unwrap();
        let result = remove_role_from_person(&pool, person_id, role_id).await.unwrap();
        assert!(result);

        let roles = get_person_roles(&pool, person_id).await.unwrap();
        assert_eq!(roles.len(), 0);
    }

    #[tokio::test]
    async fn test_get_person_with_roles() {
        let pool = setup_test_db().await;

        let person = Person::new("Alice".to_string(), "Brown".to_string());
        let created_person = create_person(&pool, &person).await.unwrap();
        let person_id = created_person.id.unwrap();

        let role1 = Role::new("Developer".to_string(), 18.0);
        let role2 = Role::new("Manager".to_string(), 20.0);
        let created_role1 = create_role(&pool, &role1).await.unwrap();
        let created_role2 = create_role(&pool, &role2).await.unwrap();

        add_role_to_person(&pool, person_id, created_role1.id.unwrap()).await.unwrap();
        add_role_to_person(&pool, person_id, created_role2.id.unwrap()).await.unwrap();

        let person_with_roles = get_person_with_roles(&pool, person_id).await.unwrap();
        assert!(person_with_roles.is_some());

        let person_with_roles = person_with_roles.unwrap();
        assert_eq!(person_with_roles.person.name, "Alice");
        assert_eq!(person_with_roles.roles.len(), 2);
        assert!(person_with_roles.roles.contains(&"Developer".to_string()));
        assert!(person_with_roles.roles.contains(&"Manager".to_string()));
    }

    #[tokio::test]
    async fn test_set_person_roles() {
        let pool = setup_test_db().await;

        let person = Person::new("Bob".to_string(), "Green".to_string());
        let created_person = create_person(&pool, &person).await.unwrap();
        let person_id = created_person.id.unwrap();

        let role_names = vec!["Developer".to_string(), "Architect".to_string()];
        set_person_roles(&pool, person_id, &role_names).await.unwrap();

        let roles = get_person_roles(&pool, person_id).await.unwrap();
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&"Developer".to_string()));
        assert!(roles.contains(&"Architect".to_string()));

        // Verify auto-created roles have 0.0 delegation hours
        let architect_role = get_role_by_name(&pool, "Architect").await.unwrap().unwrap();
        assert_eq!(architect_role.delegation_hours, 0.0);

        // Update roles
        let new_role_names = vec!["Manager".to_string()];
        set_person_roles(&pool, person_id, &new_role_names).await.unwrap();

        let roles = get_person_roles(&pool, person_id).await.unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0], "Manager");
    }

    #[tokio::test]
    async fn test_get_all_persons_with_roles() {
        let pool = setup_test_db().await;

        let person1 = Person::new("Alice".to_string(), "Brown".to_string());
        let person2 = Person::new("Bob".to_string(), "Green".to_string());

        let created_person1 = create_person(&pool, &person1).await.unwrap();
        let created_person2 = create_person(&pool, &person2).await.unwrap();

        set_person_roles(&pool, created_person1.id.unwrap(), &vec!["Developer".to_string()]).await.unwrap();
        set_person_roles(&pool, created_person2.id.unwrap(), &vec!["Manager".to_string(), "Designer".to_string()]).await.unwrap();

        let all = get_all_persons_with_roles(&pool).await.unwrap();

        assert_eq!(all.len(), 2);
        assert_eq!(all[0].person.name, "Alice");
        assert_eq!(all[0].roles, vec!["Developer"]);
        assert_eq!(all[1].person.name, "Bob");
        assert_eq!(all[1].roles.len(), 2);
    }
}
