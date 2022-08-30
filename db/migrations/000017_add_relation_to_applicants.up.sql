ALTER TABLE "applicants" 
  ADD FOREIGN KEY("recruitment_id") 
    REFERENCES "recruitments"("id") 
    ON DELETE CASCADE, 
  ADD FOREIGN KEY("user_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE;
