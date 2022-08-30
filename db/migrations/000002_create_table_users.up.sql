CREATE TABLE IF NOT EXISTS "users"(
  "id" BIGSERIAL PRIMARY KEY, 
  "name" VARCHAR(50) NOT NULL, 
  "email" VARCHAR(100) UNIQUE NOT NULL, 
  "unverified_email" VARCHAR(100) NULL,
  "role" user_role NOT NULL DEFAULT 'general', 
  "avatar" VARCHAR NOT NULL DEFAULT 'https://abs.twimg.com/sticky/default_profile_images/default_profile.png', 
  "introduction" VARCHAR(160) NULL, 
  "email_verification_status" email_verification_status NOT NULL DEFAULT 'pending', 
  "email_verification_code" VARCHAR NULL, 
  "email_verification_code_expires_at" TIMESTAMP WITH TIME ZONE NULL, 
  "password_digest" VARCHAR NULL, 
  "password_reset_token" varchar UNIQUE NULL,
  "password_reset_token_expires_at" TIMESTAMP WITH TIME ZONE NULL,
  "last_sign_in_at" TIMESTAMP WITH TIME ZONE NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL, 
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL
);