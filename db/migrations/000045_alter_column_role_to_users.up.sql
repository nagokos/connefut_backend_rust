ALTER TABLE "users" 
  ALTER COLUMN "role" DROP DEFAULT,
  ALTER COLUMN "role"
    SET DATA TYPE user_role
    USING role::varchar::user_role,
  ALTER COLUMN "role" SET DEFAULT 'general';