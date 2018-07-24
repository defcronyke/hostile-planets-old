cd hpclient
cargo build --features gl
cd ..
cp target/debug/hpclient.dll ./hpclient_gl.pyd
