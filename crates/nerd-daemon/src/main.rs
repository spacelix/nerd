fn main() {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    if let Some(argument) = arguments.next() {
        if argument == "--version" && arguments.next().is_none() {
            println!("nerd-daemon {}", nerd_daemon::application_version());
            return;
        }
        if argument == "--security-check" && arguments.next().is_none() {
            match nerd_daemon::check_process_security() {
                Ok(()) => {
                    println!("nerd-daemon: process token is non-elevated");
                    return;
                }
                Err(error) => {
                    eprintln!("nerd-daemon: {error}");
                    std::process::exit(14);
                }
            }
        }
        eprintln!("usage: nerd-daemon [--version|--security-check]");
        std::process::exit(2);
    }

    if let Err(error) = nerd_daemon::run() {
        eprintln!("nerd-daemon: {error}");
        std::process::exit(error.exit_code());
    }
}
