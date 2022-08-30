CREATE TABLE IF NOT EXISTS "stocks"(
  "id" BIGSERIAL UNIQUE, 
  "recruitment_id" BIGINT NULL, 
  "user_id" BIGINT NULL, 
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  PRIMARY KEY("id"),
  UNIQUE("user_id", "recruitment_id")
);
CREATE INDEX ON "stocks"("user_id");
CREATE INDEX ON "stocks"("recruitment_id");
