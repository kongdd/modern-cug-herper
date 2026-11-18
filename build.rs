fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        // 启用 Task Dialog 所需的控件版本；清单同时声明 Per-Monitor V2 DPI。
        embed_manifest::embed_manifest(embed_manifest::new_manifest("CUG.CourseSetup"))
            .expect("failed to embed Windows manifest");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
