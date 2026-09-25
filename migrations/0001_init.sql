CREATE TYPE entry AS ENUM ('file', 'directory');
CREATE TABLE files (
  path text PRIMARY KEY,
  type entry NOT NULL,
  name text NOT NULL,
  parent_path text REFERENCES files,
  size bigint NOT NULL,
  content_type text NOT NULL,
  last_modified bigint NOT NULL
);
