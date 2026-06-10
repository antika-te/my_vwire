fn main() {
    // 告訴 cargo 重新編譯如果 build.rs 改變
    println!("cargo:rerun-if-changed=build.rs");
    
    // eBPF 程序需要交叉編譯到 bpf 目標
    // 這裡設置編譯提示
    println!("cargo:warning=eBPF program will be compiled separately");
}
