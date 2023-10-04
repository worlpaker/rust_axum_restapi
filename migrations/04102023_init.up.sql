-- Enable the uuid-ossp extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create status type
CREATE TYPE Status AS ENUM ('Available', 'NOTAvailable', 'Rented');

-- Create the book table with UUID primary key
CREATE TABLE IF NOT EXISTS book (
  Id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
  name varchar(255) NOT NULL UNIQUE,
  year integer NOT NULL,
  category varchar(100) NOT NULL,
  status Status NOT NULL,
  author varchar(100) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create the author table with UUID primary key
CREATE TABLE IF NOT EXISTS author (
  Id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
  name varchar(255) NOT NULL UNIQUE,
  country varchar(100) NOT NULL,
  birth_date varchar(100) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create the users table with UUID primary key
CREATE TABLE IF NOT EXISTS users (
  Id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
  name varchar(100) NOT NULL,
  nation_id varchar(100) NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create the users_history table with UUID primary key
CREATE TABLE IF NOT EXISTS users_history (
  Id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
  nation_id varchar(100) NOT NULL,
  book_name varchar(255) NOT NULL,
  due_date varchar(100) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- Add foreign key constraints
ALTER TABLE book
ADD FOREIGN KEY (author) REFERENCES author(name);

ALTER TABLE users_history
ADD FOREIGN KEY (book_name) REFERENCES book(name),
ADD FOREIGN KEY (nation_id) REFERENCES users(nation_id);


-- Create the trigger function
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at := NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create the trigger for the book table
CREATE TRIGGER set_timestamp_book
BEFORE UPDATE ON book
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();

-- Create the trigger for the author table
CREATE TRIGGER set_timestamp_author
BEFORE UPDATE ON author
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();

-- Create the trigger for the users table
CREATE TRIGGER set_timestamp_users
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();
