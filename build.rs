fn main() -> Result<(), embed_resource::CompilationResult> {
    println!("cargo:rerun-if-changed=assets/mira.rc");
    println!("cargo:rerun-if-changed=assets/mira.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile_for(
            "assets/mira.rc",
            ["mira"],
            embed_resource::ParamsIncludeDirs(["assets"]),
        )
        .manifest_required()?;
    }
    Ok(())
}
