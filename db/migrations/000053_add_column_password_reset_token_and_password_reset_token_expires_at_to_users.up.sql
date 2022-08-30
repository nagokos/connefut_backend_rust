ALTER TABLE "users"
  ADD COLUMN "password_reset_token" VARCHAR NULL UNIQUE,
  ADD COLUMN "password_reset_token_expires_at" TIMESTAMP WITH TIME ZONE NULL;