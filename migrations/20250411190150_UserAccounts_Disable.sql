-- We assume that users use SQLite >= 3.25.0
ALTER TABLE UserAccounts RENAME COLUMN tls TO trust_invalid_certs ;

ALTER TABLE UserAccounts ADD COLUMN enabled INTEGER DEFAULT 1 ;
