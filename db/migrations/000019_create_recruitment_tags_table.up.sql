CREATE TABLE IF NOT EXISTS "recruitment_tags"(
  "id" BIGSERIAL UNIQUE,
  "recruitment_id" BIGINT NULL, 
  "tag_id" BIGINT NULL, 
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  PRIMARY KEY("id"),
  UNIQUE("recruitment_id", "tag_id")
);
CREATE INDEX ON "recruitment_tags"("tag_id");
CREATE INDEX ON "recruitment_tags"("recruitment_id");