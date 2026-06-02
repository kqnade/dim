use dim::app::App;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = args.get(1).map(std::path::PathBuf::from);

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        original_hook(info);
    }));

    match App::new(file_path) {
        Ok(mut app) => {
            if let Err(e) = app.run() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize: {}", e);
            std::process::exit(1);
        }
    }
}
