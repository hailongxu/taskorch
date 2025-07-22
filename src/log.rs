#![allow(dead_code)]

#[allow(unused_imports)]
use std::fmt::Display;
use std::str::{from_utf8, from_utf8_unchecked};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd)]
pub(crate) enum Level {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Log level options (mutually exclusive - select one or none)
/// When multiple are selected, compilation will fail.
pub(crate) const LEVEL: Level = {
    #[cfg(feature = "log-error")]
    const MAX_LEVEL: Level = Level::Error;
    #[cfg(feature = "log-warn")]
    const MAX_LEVEL: Level = Level::Warn;
    #[cfg(feature = "log-info")]
    const MAX_LEVEL: Level = Level::Info;
    #[cfg(feature = "log-debug")]
    const MAX_LEVEL: Level = Level::Debug;
    #[cfg(feature = "log-trace")]
    const MAX_LEVEL: Level = Level::Trace;
    #[cfg(not(any(
        feature = "log-error",
        feature = "log-warn",
        feature = "log-info",
        feature = "log-debug",
        feature = "log-trace",
    )))]
    const MAX_LEVEL: Level = Level::Off;
    MAX_LEVEL
};

pub(crate) const COLOR_RED: &'static str= "\x1b[31m";
pub(crate) const COLOR_YELLOW: &'static str = "\x1b[33m";
pub(crate) const COLOR_GREEN: &'static str = "\x1b[32m";
pub(crate) const COLOR_WHITE: &'static str = "\x1b[37m";
pub(crate) const COLOR_BLUE: &'static str = "\x1b[34m";
pub(crate) const COLOR_CYAN: &'static str = "\x1b[36m";
pub(crate) const COLOR_GREY: &'static str = "\x1b[90m";
pub(crate) const COLOR_ESCAPE: &'static str= "\x1b[0m";

#[cfg(feature="log-color")]
pub(crate) mod level_color {
    use super::*;
    pub(crate) const COLOR_ERROR: &'static str= COLOR_RED;
    pub(crate) const COLOR_WARN: &'static str = COLOR_YELLOW;
    pub(crate) const COLOR_INFO: &'static str = COLOR_GREEN;
    pub(crate) const COLOR_DEBUG: &'static str = COLOR_BLUE;
    pub(crate) const COLOR_TRACE: &'static str = COLOR_GREY;
    pub(crate) const COLOR_END: &'static str = COLOR_ESCAPE;
}

#[cfg(not(feature="log-color"))]
pub(crate) mod level_color {
    pub(crate) const COLOR_ERROR: &'static str= "";
    pub(crate) const COLOR_WARN: &'static str = "";
    pub(crate) const COLOR_INFO: &'static str = "";
    pub(crate) const COLOR_DEBUG: &'static str = "";
    pub(crate) const COLOR_TRACE: &'static str = "";
    pub(crate) const COLOR_END: &'static str = "";
}

#[allow(unused_imports)]
pub(crate) use level_color::*;

static START_TIME: OnceLock<Instant> = OnceLock::new();
pub(crate) static LOG_GLOBAL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
pub(crate) const NAME:&'static str = "taskorch";

#[allow(unused_macros)]
macro_rules! log {
    ($level:literal $color_head:expr, $color_tail:expr; $($args:tt)*) => {{
        let msg = log_str!($level $color_head, $color_tail; $($args)*);
        let lock = crate::log::LOG_GLOBAL_LOCK.get_or_init(||::std::sync::Mutex::new(()));
        let _guard = lock.lock();
        log_print!("{}",msg)
    }}
}

#[allow(unused_macros)]
macro_rules! log_str {
    ($level:literal $color_head:expr, $color_tail:expr; $($args:tt)*) => {{
        let color_head = $color_head;
        let color_tail = $color_tail;
        let level = $level;
        let ts = crate::log::uptime();
        let mut thid_buff = [0u8;32];
        let (thid_head,thid_tail) = crate::log::format_concise_current_threadid(&mut thid_buff);
        let filepre = myfilepre!();
        let file = myfile!();
        let (linepre,line) = myline!();
        let name = crate::log::NAME;

        format!(
            "{color_head}[{ts} \
            {level:<5} \
            {name}\
            {filepre}{file}{linepre}{line} \
            {thid_head}{thid_tail}]\
            {color_tail} {}",
            format!($($args)*))
    }}
}


#[test]
fn test_format_by_mutstr() {
    // it does not work
    let mut tid_buf = [0u8; 16];
    if let Ok(_s) = std::str::from_utf8_mut(&mut tid_buf) {
        #[cfg(false)]
        // s is err
        if write!(s, "{}", "").is_err() {
            // buffer is unsufficient
        }
    }
}

#[allow(unused)]
#[cfg(any(
    feature = "log-error",
    feature = "log-warn",
))]
macro_rules! log_print {
    ($($args:tt)*) => {
        eprintln!($($args)*)
    };
}

#[allow(unused)]
#[cfg(not(any(
    feature = "log-error",
    feature = "log-warn",
)))]
macro_rules! log_print {
    ($($args:tt)*) => {
        println!($($args)*)
    };
}


#[allow(unused)]
#[cfg(true)]
// #[cfg(not(any(
//     feature="log-file",
//     feature="log-line"
// )))]
macro_rules! myfilepre {
    () => {
        ""
    };
}

#[allow(unused)]
#[cfg(false)]
// #[cfg(any(
//     feature="log-file",
//     feature="log-line"
// ))]
macro_rules! myfilepre {
    () => {
        " "
    };
}

#[allow(unused)]
#[cfg(true)]
// #[cfg(not(feature="log-file"))]
macro_rules! myfile {
    () => {
        ""
    };
}
#[cfg(false)]
#[allow(unused)]
// #[cfg(feature="log-file")]
macro_rules! myfile {
    () => {
        file!()
    };
}


#[allow(unused_macros)]
#[cfg(true)]
// #[cfg(not(feature="log-line"))]
macro_rules! myline {
    () => {
        ("","")
    };
}
#[allow(unused_macros)]
#[cfg(false)]
// #[cfg(feature="log-line")]
macro_rules! myline {
    () => {
        (":",line!())
    };
}

// error level

#[allow(unused_macros)]
#[cfg(not(any(
    feature = "log-error",
    feature = "log-warn",
    feature = "log-info",
    feature = "log-debug",
    feature = "log-trace",
)))]
macro_rules! error {
    ($($args:tt)*) => {
        {}
    };
}

#[allow(unused_macros)]
#[cfg(any(
    feature = "log-error",
    feature = "log-warn",
    feature = "log-info",
    feature = "log-debug",
    feature = "log-trace",
))]
macro_rules! error {
    ($($args:tt)*) => {
        log!("error" crate::log::COLOR_ERROR, crate::log::COLOR_END; $($args)*)
    };
}

// wran level

#[allow(unused_macros)]
#[cfg(not(any(
    feature = "log-warn",
    feature = "log-info",
    feature = "log-debug",
    feature = "log-trace",
)))]
macro_rules! warn {
    ($($args:tt)*) => {
        {}
    };
}

#[allow(unused_macros)]
#[cfg(any(
    feature = "log-warn",
    feature = "log-info",
    feature = "log-debug",
    feature = "log-trace",
))]
macro_rules! warn {
    ($($args:tt)*) => {
        log!("warn" crate::log::COLOR_WARN, crate::log::COLOR_END; $($args)*)
    };
}


// info level
#[allow(unused_macros)]
#[cfg(not(any(
    feature = "log-info",
    feature = "log-debug",
    feature = "log-trace",
)))]
macro_rules! info {
    ($($args:tt)*) => {
        {}
    };
}

#[allow(unused_macros)]
#[cfg(any(
    feature = "log-info",
    feature = "log-debug",
    feature = "log-trace",
))]
macro_rules! info {
    ($($args:tt)*) => {
        log!("info" crate::log::COLOR_INFO, crate::log::COLOR_END; $($args)*)
    };
}


// debug level

#[allow(unused_macros)]
#[cfg(not(any(
    feature = "log-debug",
    feature = "log-trace",
)))]
macro_rules! debug {
    ($($args:tt)*) => {
        {}
    };
}

#[allow(unused_macros)]
#[cfg(any(
    feature = "log-debug",
    feature = "log-trace",
))]
macro_rules! debug {
    ($($args:tt)*) => {
        log!("debug" crate::log::COLOR_DEBUG, crate::log::COLOR_END; $($args)*)
    };
}


// trace level

#[allow(unused_macros)]
#[cfg(not(any(
    feature = "log-trace",
)))]
macro_rules! trace {
    ($($args:tt)*) => {
        {}
    };
}

#[allow(unused_macros)]
#[cfg(any(
    feature = "log-trace",
))]
macro_rules! trace {
    ($($args:tt)*) => {
        log!("trace" crate::log::COLOR_TRACE, crate::log::COLOR_END; $($args)*)
    };
}

pub(crate) fn init_starttime() {
    START_TIME.get_or_init(::std::time::Instant::now);
}
pub(crate) struct Timespan {
    day: u32,
    hour: u8,
    min: u8,
    sec: u8,
    micro: u32,
}

pub(crate) fn uptime()->Timespan {
    let start = START_TIME.get_or_init(::std::time::Instant::now);
    let e = start.elapsed();
    let s = e.as_secs();
    let day = (s/(3600*24)) as u32;
    let s = s%(3600*24);
    let hour = (s/3600) as u8;
    let s = s%3600;
    let min = (s/60) as u8;
    let sec = (s%60) as u8;
    let micro = e.subsec_micros() as u32;
    Timespan { day, hour, min, sec, micro }
}

impl std::fmt::Display for Timespan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Timespan {day:0,hour:0,min:0,sec:s,micro:ss} => {
                write!(f,"{s:02}.{ss:06}")
            }
            Timespan {day:0,hour:0,min:m,sec:s,micro:ss} => {
                write!(f,"{m:02}:{s:02}.{ss:06}")
            }
            Timespan {day:0,hour:h,min:m,sec:s,micro:ss} => {
                write!(f,"{h:02}:{m:02}:{s:02}.{ss:06}")
            }
            Timespan {day:d,hour:h,min:m,sec:s,micro:ss} => {
                write!(f,"{d}.{h:02}:{m:02}:{s:02}.{ss:06}")
            }
        }
    }
}


#[allow(unused_macros)]
macro_rules! sleep_millis {
    () => {
        ::std::thread::sleep(std::time::Duration::from_millis(1));
    };
    ($t:expr) => {
        ::std::thread::sleep(std::time::Duration::from_millis($t));
    };
}


#[test]
fn test_log() {
    error!("message error");
    warn!("message warn");
    info!("message info");
    sleep_millis!(10);
    debug!("message debug");
    trace!("message trace");
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

// #[warn(unused_assignments)]
fn concise_threadid(buf:&[u8;32], len:usize)->(&str,&str) {
    debug_assert!(len <= buf.len());
    const HEAD: [u8; 9] = [b'T',b'h',b'r',b'e',b'a',b'd',b'I',b'd',b'('];
    let head_verified = buf.starts_with(&HEAD);
    let tail_verified = buf[len-1] == b')';
    let (head, tail): (&str,&str);
    if head_verified && tail_verified {
        head = from_utf8(&buf[..2]).unwrap_or("");
        unsafe {
        tail = from_utf8_unchecked(&buf[HEAD.len()-3..len]);
        }
    } else {
        head = from_utf8(&buf[0..len]).unwrap_or("unknown-thereadId-format");
        tail = "";
    }
    (head, tail)
}

#[allow(unused)]
type ThreadIdBuf = [u8;32];
#[allow(dead_code)]
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

    format_concise_current_threadid(&mut buf);
}

#[test]
fn test_format() {
    let _a = format_args!("hello {}", "world");
    let _a = std::fmt::format(format_args!("hello {}", "world"));
}

#[test]
fn test_color() {
    println!("\x1b[31merror is red text\x1b[0m",);
    println!("\x1b[33mwarn is yellow text\x1b[0m");
    println!("\x1b[32minfo is std green\x1b[0m"); 
    println!("\x1b[92minfo is bright green\x1b[0m");
    println!("\x1b[34m[DEBUG] is std blue\x1b[0m");
    println!("\x1b[94m[DEBUG] is bright blue\x1b[0m");
    println!("\x1b[90m[DEBUG] is grey\x1b[0m");
    println!("\x1b[36mtrace is std cyan\x1b[0m");
    println!("\x1b[96mtrace is bright cyan\x1b[0m");
}

#[test]
fn test_format_write() {
    let mut ts = Timespan {
        day: 1,
        hour: 1,
        min: 2,
        sec: 3,
        micro: 4,
    };
    println!("{ts}");
    ts.day = 0;
    println!("{ts}");
    ts.hour = 0;
    println!("{ts}");
    ts.min = 0;
    println!("{ts}");
}