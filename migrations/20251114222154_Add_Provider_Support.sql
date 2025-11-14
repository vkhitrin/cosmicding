CREATE TABLE UserAccounts_new (
    id INTEGER PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL,
    instance TEXT NOT NULL,
    api_token TEXT NOT NULL,
    last_sync_status INTEGER NOT NULL,
    last_sync_timestamp INTEGER NOT NULL,
    trust_invalid_certs INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    enable_sharing INTEGER NOT NULL DEFAULT 0,
    enable_public_sharing INTEGER NOT NULL DEFAULT 0,
    provider TEXT NOT NULL,
    provider_version TEXT,
    CHECK (
        trust_invalid_certs IN (0, 1)
        AND last_sync_status IN (0, 1)
        AND enabled IN (0, 1)
        AND enable_sharing IN (0, 1)
        AND enable_public_sharing IN (0, 1)
    )
);

INSERT INTO UserAccounts_new (
    id,
    display_name,
    instance,
    api_token,
    last_sync_status,
    last_sync_timestamp,
    trust_invalid_certs,
    enabled,
    enable_sharing,
    enable_public_sharing,
    provider
)
SELECT
    id,
    display_name,
    instance,
    api_token,
    last_sync_status,
    last_sync_timestamp,
    trust_invalid_certs,
    enabled,
    enable_sharing,
    enable_public_sharing,
    'linkding'
FROM UserAccounts;

INSERT INTO UserAccounts_new (
    display_name,
    instance,
    api_token,
    last_sync_status,
    last_sync_timestamp,
    trust_invalid_certs,
    enabled,
    enable_sharing,
    enable_public_sharing,
    provider
)
SELECT
    'Cosmicding',
    '',
    '',
    1,
    0,
    0,
    1,
    0,
    0,
    'cosmicding'
WHERE NOT EXISTS (SELECT 1 FROM UserAccounts_new WHERE provider = 'cosmicding');

DROP TABLE UserAccounts;
ALTER TABLE UserAccounts_new RENAME TO UserAccounts;

CREATE TABLE Bookmarks_new (
    id INTEGER PRIMARY KEY NOT NULL,
    user_account_id INTEGER NOT NULL,
    provider_internal_id INTEGER,
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
    website_description TEXT,
    is_owner INTEGER DEFAULT 1,
    CHECK (
        is_archived IN (0, 1)
        AND unread IN (0, 1)
        AND shared IN (0, 1)
    )
);

INSERT INTO Bookmarks_new (
    id,
    user_account_id,
    provider_internal_id,
    url,
    title,
    description,
    notes,
    web_archive_snapshot_url,
    favicon_url,
    preview_image_url,
    is_archived,
    unread,
    shared,
    tag_names,
    date_added,
    date_modified,
    website_title,
    website_description,
    is_owner
)
SELECT
    id,
    user_account_id,
    linkding_internal_id,
    url,
    title,
    description,
    notes,
    web_archive_snapshot_url,
    favicon_url,
    preview_image_url,
    is_archived,
    unread,
    shared,
    tag_names,
    date_added,
    date_modified,
    website_title,
    website_description,
    is_owner
FROM Bookmarks;

DROP TABLE Bookmarks;
ALTER TABLE Bookmarks_new RENAME TO Bookmarks;
