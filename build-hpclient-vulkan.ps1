cd hpclient
cargo build --no-default-features --features vulkan --target-dir ../target-vulkan
cd ..
cp target-vulkan/debug/hpclient.dll ./hpclient_vulkan.pyd
