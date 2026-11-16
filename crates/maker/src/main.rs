mod app;
mod art;
mod work;

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--write-sample") {
        let dir = args.get(1).map(String::as_str).unwrap_or("sample");
        if let Err(err) = work::write_project(std::path::Path::new(dir), None) {
            eprintln!("{err}");
            std::process::exit(1);
        }
        std::process::exit(0);
    }
    if args.first().map(String::as_str) == Some("--build") {
        let dir = std::path::PathBuf::from(args.get(1).map(String::as_str).unwrap_or("sample"));
        match work::assemble(&dir) {
            Ok(dest) => println!("{}", dest.display()),
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }
    app::start()
}
