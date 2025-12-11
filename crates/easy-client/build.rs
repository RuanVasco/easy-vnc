fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();

        res.set_manifest_file("assets/windows.manifest");

        match res.compile() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Erro ao compilar recursos do Windows: {}", e);
                std::process::exit(1);
            }
        }
    }
}
