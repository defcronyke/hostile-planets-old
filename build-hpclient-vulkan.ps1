cd hpclient
cargo build --features vulkan
cd ..
cp target/debug/hpclient.dll ./hpclient_vulkan.pyd
