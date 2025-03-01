-- Workaround due to linkding not exposing the bookmark owner account via the API
ALTER TABLE Bookmarks ADD COLUMN is_owner INTEGER DEFAULT 1;
