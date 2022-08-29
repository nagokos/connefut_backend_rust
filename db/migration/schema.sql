--* users table
CREATE TABLE IF NOT EXISTS "users"(
  "id" BIGSERIAL PRIMARY KEY, 
  "name" VARCHAR(50) NOT NULL, 
  "email" VARCHAR(100) UNIQUE NOT NULL, 
  "role" VARCHAR NOT NULL DEFAULT 'general', 
  "avatar" VARCHAR NOT NULL DEFAULT 'https://abs.twimg.com/sticky/default_profile_images/default_profile.png', 
  "introduction" VARCHAR(160) NULL, 
  "email_verification_status" VARCHAR NOT NULL DEFAULT 'pending', 
  "email_verification_code" VARCHAR NULL, 
  "email_verification_code_expires_at" TIMESTAMP WITH TIME ZONE NULL, 
  "password_digest" VARCHAR NULL, 
  "last_sign_in_at" TIMESTAMP WITH TIME ZONE, 
  "unverified_email" VARCHAR(100),
  "password_reset_token" VARCHAR,
  "password_reset_token_expires_at" TIMESTAMP WITH TIME ZONE,
  "website_url" VARCHAR,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL
);
CREATE INDEX ON "users"("email_verification_code");

--* prefectures table
CREATE TABLE IF NOT EXISTS "prefectures"(
  "id" BIGSERIAL PRIMARY KEY, 
  "name" VARCHAR NOT NULL UNIQUE, 
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL
);

--* sports table
CREATE TABLE IF NOT EXISTS "sports"(
  "id" BIGSERIAL PRIMARY KEY, 
  "name" VARCHAR NOT NULL UNIQUE, 
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL
);

--* tags table
CREATE TABLE IF NOT EXISTS "tags"(
  "id" BIGSERIAL PRIMARY KEY, 
  "name" VARCHAR UNIQUE NOT NULL, 
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL
);
