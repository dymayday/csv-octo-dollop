mod engine;
use engine::Engine;
use std::env;

fn main() {
    // Command line handling part to end up with a path
    // to the transaction file.
    let path_csv = {
        let mut args = env::args();
        match args.len() {
            1 => {
                // No argument passed.
                eprintln!("Please feed me with a transactions file as command line argument.");
                std::process::exit(1)
            }
            2 => args.nth(1).expect("Fail to read command line argument."),
            _ => {
                // Too many argument entered.
                eprintln!(
                    "Please feed me with only one transactions file as command line argument."
                );
                std::process::exit(1)
            }
        }
    };

    let mut engine = Engine::new();
    match engine.process(&path_csv) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Engine failed with error : {}.", e);
            std::process::exit(1)
        }
    }

    engine.print_db();
}
