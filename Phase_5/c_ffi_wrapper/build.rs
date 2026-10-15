fn main() {
    println!("cargo:rerun-if-changed=c_src/crypto_ops.c");
    println!("cargo:rerun-if-changed=c_src/crypto_ops.h");

    cc::Build::new()
        .file("c_src/crypto_ops.c")
        .include("c_src")
        .compile("crypto_ops");
}
