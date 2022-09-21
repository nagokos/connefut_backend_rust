CREATE TABLE IF NOT EXISTS "recruitment_tags"(
  "id" BIGSERIAL PRIMARY KEY,
  "tag_id" BIGINT NOT NULL,
  "recruitment_id" BIGINT NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  FOREIGN KEY("tag_id") 
    REFERENCES "tags"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("recruitment_id") 
    REFERENCES "recruitments"("id")
    ON DELETE CASCADE,
  UNIQUE("tag_id", "recruitment_id")
);
CREATE INDEX ON "recruitment_tags"("tag_id");
CREATE INDEX ON "recruitment_tags"("recruitment_id");
