cd hpclient
cargo build --release --no-default-features --features gl --target-dir ../target-gl
cd ..
cp target-gl/release/hpclient.dll ./hpclient_gl.pyd
