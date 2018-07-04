#!/usr/bin/env bash

pg_dump \
  --dbname="$(heroku config:get DATABASE_URL -a mscbot-rs)" \
  --inserts \
  --data-only \
  --table=githubuser \
  > githubuser-backup.pg
