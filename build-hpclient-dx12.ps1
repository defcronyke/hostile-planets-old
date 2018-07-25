cd hpclient
cargo build --no-default-features --features dx12 --target-dir ../target-dx12
cd ..
cp target-dx12/debug/hpclient.dll ./hpclient_dx12.pyd
