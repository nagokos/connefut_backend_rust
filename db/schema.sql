CREATE TABLE IF NOT EXISTS "users"(
  "id" BIGSERIAL UNIQUE, 
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
  "unverified_email" VARCHAR(160),
  "password_reset_token" VARCHAR,
  "password_reset_token_expires_at" TIMESTAMP WITH TIME ZONE,
  "website_url" VARCHAR,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  PRIMARY KEY("id")
);
CREATE INDEX ON "users"("email_verification_code");

