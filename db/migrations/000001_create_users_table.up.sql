CREATE TABLE IF NOT EXISTS "users"(
  "id" BIGSERIAL UNIQUE, 
  "name" VARCHAR(50) NOT NULL, 
  "email" VARCHAR(100) UNIQUE NOT NULL, 
  "role" VARCHAR NOT NULL DEFAULT 'general', 
  "avatar" VARCHAR NOT NULL DEFAULT 'https://abs.twimg.com/sticky/default_profile_images/default_profile.png', 
  "introduction" VARCHAR(4000) NULL, 
  "email_verification_status" VARCHAR NOT NULL DEFAULT 'pending', 
  "email_verification_token" VARCHAR NULL, 
  "email_verification_token_expires_at" TIMESTAMP WITH TIME ZONE NULL, 
  "password_digest" VARCHAR NULL, 
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  PRIMARY KEY("id")
);
CREATE INDEX ON "users"("email_verification_token");

