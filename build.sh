cargo build -release
cp -r ./target/release/rssust ./env
./env/rssust cookie firefox
./env/rssust docs
./env/rssust