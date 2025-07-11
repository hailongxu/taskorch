use std::str::{from_utf8, from_utf8_unchecked};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

pub(crate) enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Log level options (mutually exclusive - select one or none)
/// When multiple are selected, compilation will fail.
pub(crate) const LEVEL: Option<Level> = {
    #[cfg(feature = "release_max_level_error")]
    const MAX_LEVEL: Option<Level> = Some(Level::Error);
    #[cfg(feature = "release_max_level_warn")]
    const MAX_LEVEL: Option<Level> = Some(Level::Warn);
    #[cfg(feature = "release_max_level_info")]
    const MAX_LEVEL: Option<Level> = Some(Level::Info);
    #[cfg(feature = "release_max_level_debug")]
    const MAX_LEVEL: Option<Level> = Some(Level::Debug);
    #[cfg(feature = "release_max_level_trace")]
    const MAX_LEVEL: Option<Level> = Some(Level::Trace);
    #[cfg(not(any(
        feature = "release_max_level_error",
        feature = "release_max_level_warn",
        feature = "release_max_level_info",
        feature = "release_max_level_debug",
        feature = "release_max_level_trace",
    )))]  // None level is selected
    const MAX_LEVEL: Option<Level> = None;
    MAX_LEVEL
};

static START_TIME: OnceLock<Instant> = OnceLock::new();
pub(crate) static LOG_GLOBAL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

macro_rules! log {
    ($level:literal $($args:tt)*) => {{
        let msg = log_str!($level $($args)*);
        let lock = crate::log::LOG_GLOBAL_LOCK.get_or_init(||::std::sync::Mutex::new(()));
        let _guard = lock.lock();
        log_print!("{}",msg)
    }}
}

macro_rules! log_str {
    ($level:literal $($args:tt)*) => {{
        let level = $level;
        let (s,m) = crate::log::uptime();
        let mut thid_buff = [0u8;32];
        let (thid_head,thid_tail) = crate::log::format_concise_current_threadid(&mut thid_buff);
        let filepre = myfilepre!();
        let file = myfile!();
        let (linepre,line) = myline!();
        format!(
            "[{level} {s}.{m:03}{filepre}{file}{linepre}{line} {thid_head}{thid_tail}] {}",
            format!($($args)*))
    }}
}


#[test]
fn test_format_by_mutstr() {
    // it does not work
    let mut tid_buf = [0u8; 16];
    if let Ok(s) = std::str::from_utf8_mut(&mut tid_buf) {
        #[cfg(false)]
        // s is err
        if write!(s, "{}", "").is_err() {
            // buffer is unsufficient
        }
    }
}

#[cfg(any(
    feature = "release_max_level_error",
    feature = "release_max_level_warn",
))]
macro_rules! log_print {
    ($($args:tt)*) => {
        eprintln!($($args)*)
    };
}

#[cfg(not(any(
    feature = "release_max_level_error",
    feature = "release_max_level_warn",
)))]
macro_rules! log_print {
    ($($args:tt)*) => {
        println!($($args)*)
    };
}



#[cfg(not(any(
    feature="log_file",
    feature="log_line"
)))]
macro_rules! myfilepre {
    () => {
        ""
    };
}

#[cfg(any(
    feature="log_file",
    feature="log_line"
))]
macro_rules! myfilepre {
    () => {
        " "
    };
}


#[cfg(not(feature="log_file"))]
macro_rules! myfile {
    () => {
        ""
    };
}
#[cfg(feature="log_file")]
macro_rules! myfile {
    () => {
        file!()
    };
}

#[cfg(not(feature="log_line"))]
macro_rules! myline {
    () => {
        ("","")
    };
}
#[cfg(feature="log_line")]
macro_rules! myline {
    () => {
        (":",line!())
    };
}

// for testing macro visibility just in this crate
macro_rules! this_test {
    ($str:expr) => {
        {}
    };
}

// error level

#[cfg(not(any(
    feature = "release_max_level_error",
    feature = "release_max_level_warn",
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
)))]
macro_rules! error {
    ($($args:tt)*) => {
        {}
    };
}

#[cfg(any(
    feature = "release_max_level_error",
    feature = "release_max_level_warn",
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
))]
macro_rules! error {
    ($($args:tt)*) => {
        log!("error" $($args)*)
    };
}

#[cfg(not(any(
    feature = "release_max_level_error",
    feature = "release_max_level_warn",
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
)))]
macro_rules! error_str {
    ($($args:tt)*) => {
        ""
    };
}

#[cfg(any(
    feature = "release_max_level_error",
    feature = "release_max_level_warn",
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
))]
macro_rules! error_str {
    ($($args:tt)*) => {
        log!("error" $($args)*)
    };
}


// wran level

#[cfg(not(any(
    feature = "release_max_level_warn",
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
)))]
macro_rules! warn {
    ($($args:tt)*) => {
        {}
    };
}

#[cfg(any(
    feature = "release_max_level_warn",
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
))]
macro_rules! warn {
    ($($args:tt)*) => {
        log!("warn" $($args)*)
    };
}


// info level

#[cfg(not(any(
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
)))]
macro_rules! info {
    ($($args:tt)*) => {
        {}
    };
}

#[cfg(any(
    feature = "release_max_level_info",
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
))]
macro_rules! info {
    ($($args:tt)*) => {
        log!("info" $($args)*)
    };
}


// debug level

#[cfg(not(any(
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
)))]
macro_rules! debug {
    ($($args:tt)*) => {
        {}
    };
}

#[cfg(any(
    feature = "release_max_level_debug",
    feature = "release_max_level_trace",
))]
macro_rules! debug {
    ($($args:tt)*) => {
        log!("debug" $($args)*)
    };
}


// trace level

#[cfg(not(any(
    feature = "release_max_level_trace",
)))]
macro_rules! trace {
    ($($args:tt)*) => {
        {}
    };
}

#[cfg(any(
    feature = "release_max_level_trace",
))]
macro_rules! trace {
    ($($args:tt)*) => {
        log!("trace" $($args)*)
    };
}



pub(crate) fn uptime()->(u64,u32) {
    let start = START_TIME.get_or_init(::std::time::Instant::now);
    let e = start.elapsed();
    let a = e.as_secs();
    let b = e.subsec_millis();
    (a,b)
}

macro_rules! sleep_millis {
    () => {
        ::std::thread::sleep(std::time::Duration::from_millis(1));
    };
    ($t:expr) => {
        ::std::thread::sleep(std::time::Duration::from_millis($t));
    };
}


#[test]
fn test_info() {
    error!("error");
    warn!("warn");
    info!("info");
    sleep_millis!(10);
    debug!("debug");
    trace!("trace");
}

#[test]
fn test_log_str() {
    let a = log_str!("error" "");
}

fn format_threadid(buf:&mut[u8;32])->usize {
    use std::io::{Cursor,Write};
    let mut cursor = Cursor::new(&mut buf[..]);
    let start_pos = cursor.position(); // before is 0
    
    match write!(&mut cursor, "{:?}", ::std::thread::current().id()) {
        Ok(_) => {},
        // Error: Fill the existing space and discard any excess parts
        Err(_) => error!("ThreadId conversion failed - 32-byte buffer insufficient"),
    } 

    let end_pos = cursor.position();
    let bytes_written = end_pos - start_pos;
    bytes_written as usize
}

fn concise_threadid(buf:&[u8;32], len:usize)->(&str,&str) {
    debug_assert!(len <= buf.len());
    const HEAD: [u8; 9] = [b'T',b'h',b'r',b'e',b'a',b'd',b'I',b'd',b'('];
    let head_verified = buf.starts_with(&HEAD);
    let tail_verified = buf[len-1] == b')';
    let (mut head, mut tail) = ("","");
    if head_verified && tail_verified {
        head = from_utf8(&buf[..2]).unwrap_or("");
        unsafe {
        tail = from_utf8_unchecked(&buf[HEAD.len()-3..len]);
        }
    } else {
        head = from_utf8(&buf[0..len]).unwrap_or("unknown-thereadId-format");
    }
    (head, tail)
}

type ThreadIdBuf = [u8;32];
pub(crate) fn format_concise_current_threadid(buff:&mut ThreadIdBuf)->(&str,&str) {
    let len = format_threadid(buff);
    concise_threadid(buff, len)
}

#[test]
fn test_format_threadid_by_cursor() {
    let mut buf = [0u8; 32];
    let len = format_threadid(&mut buf);
    let (head,tail) = self::concise_threadid(&buf, len);
    assert!(!head.is_empty() && !tail.is_empty());
    let s = std::str::from_utf8(&buf[..len]).unwrap();
    println!("{len} {s} {head} {tail}");
}

#[test]
fn test_format() {
    let _a = format_args!("hello {}", "world");
    let _a = std::fmt::format(format_args!("hello {}", "world"));
}