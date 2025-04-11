CREATE TABLE FaviconCache (
    favicon_url TEXT PRIMARY KEY NOT NULL,
    favicon_data BLOB NOT NULL,
    last_sync_timestamp INTEGER NOT NULL
);
