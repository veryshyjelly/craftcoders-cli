//! CLI-owned output policy. Inherited child streams deliberately bypass this.
use std::fmt::Arguments;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

static NO_COLOR: AtomicBool = AtomicBool::new(false);

pub fn set_no_color(no_color: bool) {
    NO_COLOR.store(no_color, Ordering::Relaxed);
}

pub fn no_color() -> bool {
    NO_COLOR.load(Ordering::Relaxed)
}

/// Write immediately: test commands may exit without running destructors.
pub fn print(args: Arguments<'_>, stderr: bool) {
    let choice = if no_color() {
        anstream::ColorChoice::Never
    } else {
        anstream::ColorChoice::Always
    };
    let result = if stderr {
        let mut stream = anstream::AutoStream::new(io::stderr().lock(), choice);
        stream.write_fmt(args).and_then(|()| stream.flush())
    } else {
        let mut stream = anstream::AutoStream::new(io::stdout().lock(), choice);
        stream.write_fmt(args).and_then(|()| stream.flush())
    };
    if let Err(error) = result {
        if error.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        panic!("failed writing CLI output: {error}");
    }
}
