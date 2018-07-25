cd hpclient
cargo build --release --no-default-features --features vulkan --target-dir ../target-vulkan
cd ..
cp target-vulkan/release/hpclient.dll ./hpclient_vulkan.pyd
