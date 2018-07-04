CREATE TABLE teams (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    ping VARCHAR NOT NULL,
    label VARCHAR NOT NULL
);

CREATE TABLE memberships (
    id SERIAL PRIMARY KEY,
    fk_member INTEGER REFERENCES githubuser (id),
    fk_team INTEGER REFERENCES teams (id)
);

INSERT INTO teams (name, ping, label) VALUES
    ('Core', 'test-org-blabla/spec-core', 'T-core');

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'test-org-blabla/spec-core' AND (
    u.login = 'anoadragon453'
);

INSERT INTO githubuser (id, login) VALUES
    (0, 'anoadragon453');