// Logging macros with color + timestamp

#[macro_export]
macro_rules! log_info {
    ($($x:tt)*) => {{
        use chrono::Local;
        let now = Local::now().format("%H:%M:%S");
        println!("\x1b[32m[INFO] [{}] {}\x1b[0m", now, format!($($x)*));
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        use chrono::Local;
        let now = Local::now().format("%H:%M:%S");
        println!("\x1b[33m[WARN] [{}] {}\x1b[0m", now, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        use chrono::Local;
        let now = Local::now().format("%H:%M:%S");
        eprintln!("\x1b[31m[ERROR] [{}] {}\x1b[0m", now, format!($($arg)*));
    }};
}
