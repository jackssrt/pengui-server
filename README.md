# pengui-server

a rust rewrite of [yno-server](https://github.com/ynoproject/ynoserver), aiming to be a drop-in replacement while also being faster and more stable.

## production

### setup

these instructions can also be adapted to run the original go implementation, since that doesn't have any setup instructions

1. start downloading the game you want to host
1. build with: `just build`
1. look in `/target/release`, that's where the pengui-server executable is
1. get an instance of mariadb initialized with `/sql/init.sql` or use your already existing ynoserver database
1. create a config.yaml where you're running the executable following [the development config](https://github.com/jackssrt/pengui-server/blob/dev/config.yaml) in the root of the repo  
ie if running `./pengui-server` in `/target/release` create `/target/release/config.yaml`  
or specify path to config using `--config`  
right now it contains all the fields from ynoserver, most of which aren't used yet
just leave them empty or with the bogus values from the development config
1. run the produced executable
1. use or create an nginx config that routes traffic to the created socket in `/socket/{game_name}.socket`  
where `{game_name}` is the game_name field of the config
1. when room sockets are implemented, you will have to generate a preshared key for the client and server and put it somewhere in a .bin file
1. optional - accept docker and just start using that please

## development

### setup

- symlink a copy of yume 2kki to `pengui-server/2kki`
  - so the map (`*.lmu`) files are directly in `pengui-server/2kki/*.lmu`
- when room sockets are implemented, you will have to grab the key.bin from the wasm binary (security through obscurity, the most secure of them all /s)

### running

1. `docker compose up db -d`
1. `just run`
1. `just serve`

### useful commands

- check: `just check`
- format: `just fmt`
- fully reset database: `just devdb`
  - this will **delete the entire database** and rebuild it from scratch using `/db/init.sql`
  - when the entire database has been reverse engineered we will switch to proper sqlx migrations

## license

this project is bound by the AGPL-3.0 license because it contains a lot of ported code from the original go implementation. even though most rust projects are dual licensed under the MIT license and the Apache-2.0 license, this project is still licensed under the AGPL-3.0 because of its restrictions on derivative works.

## other projects

also check out [pengui-ball](https://github.com/AcrylonitrileButadieneStyrene/pengui-ball/tree/master), the rust rewrite of the frontend.
