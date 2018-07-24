cd hpclient
cargo build --no-default-features --features vulkan
cd ..
cp target/debug/hpclient.dll ./hpclient_vulkan.pyd
