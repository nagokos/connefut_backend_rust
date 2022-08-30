CREATE TABLE IF NOT EXISTS "recruitments"(
  "id" BIGSERIAL UNIQUE,
  "title" VARCHAR(60) NOT NULL,
  "type" VARCHAR NOT NULL,
  "level" VARCHAR NOT NULL DEFAULT 'unnecessary',
  "place" VARCHAR NULL,
  "start_at" TIMESTAMP WITH TIME ZONE NULL,
  "content" VARCHAR(10000) NULL,
  "location_url" VARCHAR NULL,
  "capacity" BIGINT NULL,
  "closing_at" TIMESTAMP WITH TIME ZONE NULL,
  "competition_id" BIGINT NOT NULL,
  "prefecture_id" BIGINT NULL,
  "user_id" BIGINT NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  PRIMARY KEY("id")
);
