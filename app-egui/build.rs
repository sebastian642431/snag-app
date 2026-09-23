fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=icon.rc");
        println!("cargo:rerun-if-changed=icons/icon.ico");
        embed_resource::compile("icon.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("could not embed the icon resource");
    }
}
