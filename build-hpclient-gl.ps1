cd hpclient
cargo build --no-default-features --features gl
cd ..
cp target/debug/hpclient.dll ./hpclient_gl.pyd
