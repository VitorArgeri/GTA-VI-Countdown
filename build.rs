fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=ui/main.slint");
    println!("cargo:rerun-if-changed=ui/components.slint");
    println!("cargo:rerun-if-changed=assets/icon.png");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    slint_build::compile("ui/main.slint")?;
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("ProductName", "GTA VI Countdown");
        resource.set("FileDescription", "GTA VI Countdown Timer");
        resource.compile()?;
    }
    Ok(())
}
