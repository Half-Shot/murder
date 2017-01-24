#!/bin/sh
protoc --python_out python-server/ --rust_out src/ --proto_path=protobufs protobufs/murder.proto
mv src/murder.rs src/pb_murder.rs
cargo build
