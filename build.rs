fn main() {
    embed_resource::compile("app.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=breeze.exe.manifest");
}
