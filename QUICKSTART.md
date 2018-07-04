# Quickstart

The steps necessary to set up mscbot from scratch are the following:

setup a [github access token](https://help.github.com/articles/creating-a-personal-access-token-for-the-command-line/) with permission `repo:public_repo`.

on debian:

```
 sudo apt install libmysqlclient-dev libsqlite3-dev libpq-dev postgresql
```

setup postgres:

```
sudo -u postgres createuser -P mscbot # prompts for password
sudo -u postgres createdb -O mscbot mscbot
```

setup and run diesel_cli:

```
cargo install diesel_cli
DATABASE_URL=postgres://mscbot:<password>@localhost/mscbot diesel migration run
```

run mscbot:

```
DATABASE_URL=postgres://mscbot:<password>@localhost/mscbot DATABASE_POOL_SIZE=20 GITHUB_ACCESS_TOKEN=<access_token> GITHUB_USER_AGENT=mscbot GITHUB_WEBHOOK_SECRETS= GITHUB_SCRAPE_INTERVAL=5 POST_COMMENTS=true cargo run
```

If you need debug logging, prepend `RUST_LOG=debug` to the above command.
