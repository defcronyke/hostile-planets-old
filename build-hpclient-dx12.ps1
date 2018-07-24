cd hpclient
cargo build --features dx12
cd ..
cp target/debug/hpclient.dll ./hpclient_dx12.pyd
