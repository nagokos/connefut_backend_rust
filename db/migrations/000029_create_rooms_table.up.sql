CREATE TABLE IF NOT EXISTS "rooms"(
  "id" BIGSERIAL UNIQUE, 
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  PRIMARY KEY("id")
);