CREATE TABLE IF NOT EXISTS "relationships"(
  "id" BIGSERIAL UNIQUE, 
  "follower_id" BIGINT NOT NULL,
  "followed_id" BIGINT NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  PRIMARY KEY("id"),
  FOREIGN KEY("follower_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("followed_id")
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  UNIQUE("follower_id", "followed_id")
);
CREATE INDEX ON "relationships"("followed_id");
CREATE INDEX ON "relationships"("follower_id");