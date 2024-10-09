CREATE TABLE IF NOT EXISTS UserAccounts (
    id INTEGER PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL,
    instance TEXT NOT NULL, 
    api_token TEXT NOT NULL,
    last_sync_status INTEGER NOT NULL,
    last_sync_timestamp INTEGER NOT NULL,
    tls INTEGER NOT NULL
    CHECK (
        tls IN (0 , 1)
        AND last_sync_status IN (0, 1)
    )
);
