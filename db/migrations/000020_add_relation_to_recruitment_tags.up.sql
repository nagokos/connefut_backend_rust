ALTER TABLE "recruitment_tags" 
  ADD FOREIGN KEY("recruitment_id") 
    REFERENCES "recruitments"("id") 
    ON DELETE CASCADE, 
  ADD FOREIGN KEY("tag_id") 
    REFERENCES "tags"("id") 
    ON DELETE CASCADE;
