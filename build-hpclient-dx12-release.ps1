cd hpclient
cargo build --release --no-default-features --features dx12 --target-dir ../target-dx12
cd ..
cp target-dx12/release/hpclient.dll ./hpclient_dx12.pyd
