fn main() {
    println!("cargo:rerun-if-env-changed=PAYLOAD_EXE");
    println!("cargo:rerun-if-env-changed=PRODUCT_NAME");
    println!("cargo:rerun-if-env-changed=EXE_NAME");
    println!("cargo:rerun-if-env-changed=APP_VERSION");

    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=icon.rc");
        println!("cargo:rerun-if-changed=icons/icon.ico");
        embed_resource::compile("icon.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("could not embed the icon resource");
    }
}
