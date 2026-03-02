# pengui-server

a rust rewrite of [yno-server](https://github.com/ynoproject/ynoserver), aiming to be a drop-in replacement while also being faster and more stable.

## installation

- build with: `just build`
- look in `/target/release`

## setup

- create a config.yaml next to the executable (or specify the path to it with `--config`) following [the development config](https://github.com/jackssrt/pengui-server/blob/dev/config.yaml) in the root of the repo

## running

1. `docker compose up db -d`
1. `just run`

## development

- check: `just check`
- format: `just fmt`
- fully reset database: `just devdb`
  - this will **delete the entire database** and rebuild it from scratch using `/db/init.sql`
  - when the entire database has been reverse engineered we will switch to proper sqlx migrations

## license

this project is bound by the AGPL-3.0 license because it contains a lot of ported code from the original go implementation. even though most rust projects are dual licensed under the MIT license and the Apache-2.0 license, this project is still licensed under the AGPL-3.0 because of its restrictions on derivative works.

## other projects

also check out [pengui-ball](https://github.com/AcrylonitrileButadieneStyrene/pengui-ball/tree/master), the rust rewrite of the frontend.
