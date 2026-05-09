-- Add startdate and enddate columns to person_roles table
-- NULL values indicate no start/end date constraint
ALTER TABLE person_roles ADD COLUMN startdate TEXT;
ALTER TABLE person_roles ADD COLUMN enddate TEXT;
