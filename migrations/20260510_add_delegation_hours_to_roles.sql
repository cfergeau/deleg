-- Add delegation_hours column to roles table
ALTER TABLE roles ADD COLUMN delegation_hours REAL NOT NULL DEFAULT 0.0;
