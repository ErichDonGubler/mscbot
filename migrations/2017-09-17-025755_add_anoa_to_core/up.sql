INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'matrix-org/spec-core' AND
    u.login = 'anoadragon453';
