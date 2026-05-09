# REST API Documentation

This document describes the REST API endpoints for the delegation hours tracking application.

## Base URL

```
http://localhost:8000
```

## Content Type

All API requests and responses use `application/json` content type.

---

## Person Endpoints

### Get All Persons

Retrieve a list of all persons with their currently active roles.

**Endpoint:** `GET /api/persons`

**Request Parameters:** None

**Response Format:**
```json
[
  {
    "id": 1,
    "name": "Jean",
    "surname": "Dupont",
    "roles": [
      {
        "role_name": "Élu titulaire CSE",
        "startdate": "2024-01-01",
        "enddate": "2026-12-31"
      },
      {
        "role_name": "Délégué syndical",
        "startdate": null,
        "enddate": null
      }
    ]
  }
]
```

**Notes:**
- Only currently active roles are returned (startdate <= today, enddate >= today or null)
- Roles with past end dates or future start dates are filtered out

---

### Get Person by ID

Retrieve a single person with their currently active roles.

**Endpoint:** `GET /api/persons/<id>`

**Path Parameters:**
- `id` (integer): The person's ID

**Response Format:**
```json
{
  "id": 1,
  "name": "Jean",
  "surname": "Dupont",
  "roles": [
    {
      "role_name": "Élu titulaire CSE",
      "startdate": "2024-01-01",
      "enddate": "2026-12-31"
    }
  ]
}
```

**Error Responses:**
- `404 Not Found`: Person with the specified ID does not exist

---

### Create Person

Create a new person with role assignments.

**Endpoint:** `POST /api/persons`

**Request Body:**
```json
{
  "name": "Alice",
  "surname": "Martin",
  "roles": [
    {
      "role_name": "Élu titulaire CSE",
      "startdate": "2024-01-01",
      "enddate": "2026-12-31"
    },
    {
      "role_name": "Délégué syndical",
      "startdate": null,
      "enddate": null
    }
  ]
}
```

**Request Fields:**
- `name` (string, required): Person's first name
- `surname` (string, required): Person's last name
- `roles` (array, required): Array of role assignments
  - `role_name` (string): Name of the role
  - `startdate` (string or null): Start date in YYYY-MM-DD format
  - `enddate` (string or null): End date in YYYY-MM-DD format

**Response Format:**
```json
{
  "id": 2,
  "name": "Alice",
  "surname": "Martin",
  "roles": [
    {
      "role_name": "Élu titulaire CSE",
      "startdate": "2024-01-01",
      "enddate": "2026-12-31"
    }
  ]
}
```

**Notes:**
- If a role doesn't exist, it will be auto-created with 0.0 delegation hours
- Returns only currently active roles (filters by dates)

---

### Update Person

Update an existing person's information and role assignments.

**Endpoint:** `PUT /api/persons/<id>`

**Path Parameters:**
- `id` (integer): The person's ID

**Request Body:**
```json
{
  "name": "Alice",
  "surname": "Martin-Dupont",
  "roles": [
    {
      "role_name": "Délégué syndical",
      "startdate": null,
      "enddate": null
    }
  ]
}
```

**Request Fields:** Same as Create Person

**Response:** `200 OK` with no body

**Error Responses:**
- `404 Not Found`: Person with the specified ID does not exist

**Notes:**
- Replaces all role assignments for the person
- To remove all roles, pass an empty `roles` array

---

### Delete Person

Delete a person and all their role assignments.

**Endpoint:** `DELETE /api/persons/<id>`

**Path Parameters:**
- `id` (integer): The person's ID

**Response:** `204 No Content`

**Error Responses:**
- `404 Not Found`: Person with the specified ID does not exist

**Notes:**
- Cascade deletes all role assignments due to foreign key constraint

---

## Role Endpoints

### Get All Roles

Retrieve a list of all roles with their delegation hours.

**Endpoint:** `GET /api/roles`

**Request Parameters:** None

**Response Format:**
```json
[
  {
    "id": 1,
    "name": "Élu titulaire CSE",
    "delegation_hours": 18.0
  },
  {
    "id": 2,
    "name": "Délégué syndical",
    "delegation_hours": 24.0
  }
]
```

---

### Get Role by ID

Retrieve a single role.

**Endpoint:** `GET /api/roles/<id>`

**Path Parameters:**
- `id` (integer): The role's ID

**Response Format:**
```json
{
  "id": 1,
  "name": "Élu titulaire CSE",
  "delegation_hours": 18.0
}
```

**Error Responses:**
- `404 Not Found`: Role with the specified ID does not exist

---

### Create Role

Create a new role.

**Endpoint:** `POST /api/roles`

**Request Body:**
```json
{
  "name": "Représentant syndical au CSE",
  "delegation_hours": 20.0
}
```

**Request Fields:**
- `name` (string, required): Role name
- `delegation_hours` (number, required): Monthly delegation hours (can be decimal)

**Response Format:**
```json
{
  "id": 3,
  "name": "Représentant syndical au CSE",
  "delegation_hours": 20.0
}
```

**Notes:**
- The `id` field in the request is optional and will be ignored
- Role names must be unique

---

### Update Role

Update an existing role's information.

**Endpoint:** `PUT /api/roles/<id>`

**Path Parameters:**
- `id` (integer): The role's ID

**Request Body:**
```json
{
  "name": "Représentant syndical au CSE",
  "delegation_hours": 22.0
}
```

**Request Fields:** Same as Create Role

**Response Format:**
```json
{
  "id": 3,
  "name": "Représentant syndical au CSE",
  "delegation_hours": 22.0
}
```

**Error Responses:**
- `404 Not Found`: Role with the specified ID does not exist

---

### Delete Role

Delete a role.

**Endpoint:** `DELETE /api/roles/<id>`

**Path Parameters:**
- `id` (integer): The role's ID

**Response:** `204 No Content`

**Error Responses:**
- `404 Not Found`: Role with the specified ID does not exist

**Notes:**
- Cascade deletes all person-role assignments using this role

---

## Web Pages

These are HTML pages rendered by the server (not JSON APIs).

### People List

Display a table of all persons with their currently active roles.

**Endpoint:** `GET /people`

**Response:** HTML page

---

### Edit Person

Display a form to edit a person's information and role assignments.

**Endpoint:** `GET /people/<id>`

**Path Parameters:**
- `id` (integer): The person's ID

**Response:** HTML page

**Error Responses:**
- `404 Not Found`: Person with the specified ID does not exist

---

### Roles List

Display a table of all roles with their delegation hours.

**Endpoint:** `GET /roles`

**Response:** HTML page

---

## Date Filtering Behavior

Role assignments include optional `startdate` and `enddate` fields. When retrieving persons:

- **Active roles** are those where:
  - `startdate` is `null` OR `startdate <= today`
  - AND `enddate` is `null` OR `enddate >= today`

- **Filtered out** roles:
  - Roles with future start dates (not yet active)
  - Roles with past end dates (expired)

All role assignments remain in the database for historical purposes, but only active ones are returned in API responses.

---

## Error Handling

### Standard Error Responses

**404 Not Found**
```
HTTP/1.1 404 Not Found
```

**500 Internal Server Error**
```
HTTP/1.1 500 Internal Server Error
```

**422 Unprocessable Entity**
```
HTTP/1.1 422 Unprocessable Entity
```
Returned when request body JSON is malformed or invalid.

---

## Examples

### Example: Create a person with multiple roles

```bash
curl -X POST http://localhost:8000/api/persons \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Marie",
    "surname": "Durand",
    "roles": [
      {
        "role_name": "Élu titulaire CSE",
        "startdate": "2024-01-01",
        "enddate": "2026-12-31"
      },
      {
        "role_name": "Délégué syndical",
        "startdate": null,
        "enddate": null
      }
    ]
  }'
```

### Example: Update a person's roles

```bash
curl -X PUT http://localhost:8000/api/persons/1 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Marie",
    "surname": "Durand",
    "roles": [
      {
        "role_name": "Représentant syndical au CSE",
        "startdate": "2026-01-01",
        "enddate": null
      }
    ]
  }'
```

### Example: Create a role

```bash
curl -X POST http://localhost:8000/api/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Membre CSSCT",
    "delegation_hours": 15.5
  }'
```

### Example: Get all persons

```bash
curl http://localhost:8000/api/persons
```

### Example: Delete a person

```bash
curl -X DELETE http://localhost:8000/api/persons/1
```
