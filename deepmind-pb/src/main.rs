fn main() {
    // let path = env::current_dir().unwrap();
    // println!("Current dir: {:?}", path);
    // let solana_proto_path = format!("{}{}", path.to_str().unwrap(), "/../proto-solana");
    // println!("solana proto path: {:?}", solana_proto_path);

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
