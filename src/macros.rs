#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!(
            "[{}] [*] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            format!($($arg)*)
        );
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!(
            "[{}] [!] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            format!($($arg)*)
        );
    };
}
