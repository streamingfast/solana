#!/bin/bash

# First:
#
#    cargo install protobuf-codegen
#
#    and ensure you have `protoc` installed.
#
# Second:
#
# you need to have the project https://github.com/streamingfast/proto-solana checked out next to this project
#

protoc --rust_out ./sdk/src/pb -I ../proto-solana sf/solana/codec/v1/codec.proto