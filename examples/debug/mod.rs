
static PROCESS_START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

#[macro_export]
macro_rules! w {
    () => {
        ::std::thread::sleep(std::time::Duration::from_millis(1));
    };
    ($t:expr) => {
        ::std::thread::sleep(std::time::Duration::from_millis($t));
    };
}

#[macro_export]
macro_rules! ti {
    () => {{
        let (s,m) = uptime();
        format!("[{s}.{m:03} {:?}]",::std::thread::current().id())
    }};
    ($s:expr) => {{
        let (s,m) = uptime();
        format!("[{s}.{m:03} {:?} {:?}]",::std::thread::current().id(),$s)
    }};
}


pub fn uptime()->(u64,u32) {
    let start = PROCESS_START_TIME.get_or_init(::std::time::Instant::now);
    let e = start.elapsed();
    let a = e.as_secs();
    let b = e.subsec_millis();
    (a,b)
}