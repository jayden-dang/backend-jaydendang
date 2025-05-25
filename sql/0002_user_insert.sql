INSERT INTO profile.users (email, username, password_hash, first_name, last_name, is_active, email_verified)
VALUES
('john.doe@example.com',     'johndoe',     'hashed_pw_123', 'John',   'Doe',     true,  true),
('jane.smith@example.com',   'janesmith',   'hashed_pw_456', 'Jane',   'Smith',   true,  false),
('alice.nguyen@example.com', 'alicenguyen', 'hashed_pw_789', 'Alice',  'Nguyen',  false, false),
('bob.le@example.com',       'boble',       'hashed_pw_321', 'Bob',    'Le',      true,  true),
('carol.tran@example.com',   'caroltran',   'hashed_pw_654', 'Carol',  'Tran',    true,  false),
('david.phan@example.com',   'davidphan',   'hashed_pw_111', 'David',  'Phan',    false, false),
('eva.ho@example.com',       'evaho',       'hashed_pw_222', 'Eva',    'Ho',      true,  true),
('frank.vo@example.com',     'frankvo',     'hashed_pw_333', 'Frank',  'Vo',      true,  true),
('grace.ly@example.com',     'gracely',     'hashed_pw_444', 'Grace',  'Ly',      false, false),
('henry.ha@example.com',     'henryha',     'hashed_pw_555', 'Henry',  'Ha',      true,  false);
