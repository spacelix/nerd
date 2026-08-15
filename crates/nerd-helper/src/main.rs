fn main() {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    if arguments.next().as_deref() == Some(std::ffi::OsStr::new("--version"))
        && arguments.next().is_none()
    {
        println!("nerd-helper {}", nerd_core::APPLICATION_VERSION);
        return;
    }

    eprintln!("nerd-helper: no privileged operations are available in Feature 01");
    eprintln!("usage: nerd-helper --version");
    std::process::exit(2);
}
