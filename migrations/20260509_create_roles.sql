-- Create roles table
CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Create person_roles junction table
CREATE TABLE IF NOT EXISTS person_roles (
    person_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    PRIMARY KEY (person_id, role_id),
    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

-- Migrate existing role data from persons table
INSERT OR IGNORE INTO roles (name)
SELECT DISTINCT role FROM persons WHERE role IS NOT NULL AND role != '';

INSERT OR IGNORE INTO person_roles (person_id, role_id)
SELECT p.id, r.id
FROM persons p
JOIN roles r ON p.role = r.name
WHERE p.role IS NOT NULL AND p.role != '';

-- Recreate persons table without role column
CREATE TABLE persons_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    surname TEXT NOT NULL
);

INSERT INTO persons_new (id, name, surname)
SELECT id, name, surname FROM persons;

DROP TABLE persons;

ALTER TABLE persons_new RENAME TO persons;
