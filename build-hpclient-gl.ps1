cd hpclient
cargo build --no-default-features --features gl --target-dir ../target-gl
cd ..
cp target-gl/debug/hpclient.dll ./hpclient_gl.pyd
