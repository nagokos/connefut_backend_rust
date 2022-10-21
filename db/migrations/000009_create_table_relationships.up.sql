CREATE TABLE IF NOT EXISTS "relationships"(
  "id" BIGSERIAL PRIMARY KEY,
  "followed_id" BIGINT NOT NULL,
  "follower_id" BIGINT NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  FOREIGN KEY("followed_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("follower_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  UNIQUE("followed_id", "follower_id")
);
CREATE INDEX ON "relationships"("followed_id");
CREATE INDEX ON "relationships"("follower_id");