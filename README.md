'cargo watch -w src -x run' for start live reloading
'cargo build --release' for build release version

you need next installed packages:

- npm vs NodeJS
- rust and cargo
- cargo-watch (cargo install cargo-watch)
- trunk (cargo install --locked trunk)

To build project you need:

1. create .env file with:
   - DB_NAME=[name].db
   - PORT=[port_number]
   - RUST_LOG=info
   - ROOT_URL=[https://example.com]
   - EXTENSIONS=.jpg,.jpeg,.png,.gif,mp4
2. create database file [name].db;
3. 'cargo build --release' for build release version;
4. build frontend 'cd frontend' and 'npm run build', it should be in 'web' folder;
5. files will be stored in 'result' folder;
