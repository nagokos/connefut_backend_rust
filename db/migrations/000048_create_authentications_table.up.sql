CREATE TABLE IF NOT EXISTS "authentications"(
  "id" BIGSERIAL UNIQUE,
  "provider" auth_provider NOT NULL,
  "uid" VARCHAR NOT NULL,
  "user_id" BIGINT NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  PRIMARY KEY("id"),
  FOREIGN KEY("user_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  UNIQUE("provider", "uid")
);
CREATE INDEX ON "authentications"("provider", "uid");
