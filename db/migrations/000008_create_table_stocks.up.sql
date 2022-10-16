CREATE TABLE IF NOT EXISTS "stocks"(
  "id" BIGSERIAL PRIMARY KEY,
  "user_id" BIGINT NOT NULL,
  "recruitment_id" BIGINT NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  FOREIGN KEY("user_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("recruitment_id") 
    REFERENCES "recruitments"("id")
    ON DELETE CASCADE,
  UNIQUE("user_id", "recruitment_id")
);
CREATE INDEX ON "stocks"("user_id");
CREATE INDEX ON "stocks"("recruitment_id");