fn main() {
    println!("cargo:rerun-if-changed=../proto");
    println!("cargo:rerun-if-changed=../proto-solana");
    tonic_build::configure()
        .out_dir("./sdk/src/pb")
        .format(true)
        .compile(
            &["sf/solana/codec/v1/codec.proto"],
            &["../proto-solana", "../proto"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile solana dm proto(s) {:?}", e));
}
