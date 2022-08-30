ALTER TABLE "users"
  RENAME COLUMN "password_reset_token" TO "password_reset_code";
ALTER TABLE "users"
  RENAME COLUMN "password_reset_token_expires_at" TO "password_reset_code_expires_at";