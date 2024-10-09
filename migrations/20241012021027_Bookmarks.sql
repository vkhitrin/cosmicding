CREATE TABLE IF NOT EXISTS Bookmarks (
    id INTEGER PRIMARY KEY NOT NULL,
    user_account_id INTEGER NOT NULL,
    linkding_internal_id INTEGER NOT NULL,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    notes TEXT,
    web_archive_snapshot_url TEXT,
    favicon_url TEXT,
    preview_image_url TEXT,
    is_archived INTEGER NOT NULL,
    unread INTEGER NOT NULL,
    shared INTEGER NOT NULL,
    tag_names TEXT,
    date_added INTEGER NOT NULL,
    date_modified INTEGER NOT NULL,
    website_title TEXT,
    website_description TEXT
    CHECK (
        is_archived IN (0 , 1)
        AND unread IN (0, 1)
        AND shared IN (0, 1)
    )
);
