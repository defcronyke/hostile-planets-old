cd hpserver
cargo build --release
cd ..
cp target/release/hpserver.dll ./hpserver.pyd
