CREATE TYPE recruitment_status AS ENUM ('draft', 'published', 'closed');
CREATE TYPE recruitment_category AS ENUM ('opponent', 'personal', 'member', 'join', 'other');

CREATE TABLE IF NOT EXISTS "recruitments"(
  "id" BIGSERIAL PRIMARY KEY,
  "title" VARCHAR(60) NOT NULL,
  "category" recruitment_category NOT NULL DEFAULT 'opponent',
  "venue" VARCHAR NULL,
  "venue_lat" DOUBLE PRECISION NULL,
  "venue_lng" DOUBLE PRECISION NULL,
  "start_at" TIMESTAMP WITH TIME ZONE NULL,
  "closing_at" TIMESTAMP WITH TIME ZONE NULL,
  "detail" VARCHAR(10000) NULL,
  "sport_id" BIGINT NOT NULL,
  "prefecture_id" BIGINT NULL,
  "status" recruitment_status NOT NULL DEFAULT 'draft',
  "user_id" BIGINT NOT NULL,
  "published_at" TIMESTAMP WITH TIME ZONE NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  FOREIGN KEY("user_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("prefecture_id") 
    REFERENCES "prefectures"("id")
    ON DELETE RESTRICT,
  FOREIGN KEY("sport_id") 
    REFERENCES "sports"("id")
    ON DELETE RESTRICT
);
CREATE INDEX ON "recruitments"("user_id");
CREATE INDEX ON "recruitments"("prefecture_id");
CREATE INDEX ON "recruitments"("sport_id");
