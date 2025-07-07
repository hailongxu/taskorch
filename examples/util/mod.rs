
static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

#[macro_export]
macro_rules! sleep_millis {
    () => {
        ::std::thread::sleep(std::time::Duration::from_millis(1));
    };
    ($t:expr) => {
        ::std::thread::sleep(std::time::Duration::from_millis($t));
    };
}

#[macro_export]
macro_rules! task_info {
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
    let start = START_TIME.get_or_init(::std::time::Instant::now);
    let e = start.elapsed();
    let a = e.as_secs();
    let b = e.subsec_millis();
    (a,b)
}

// common function for testing

// fun add with 2 param and return i32
pub(crate) fn ffadd(
    qname:&'static str, pad:&'static str, tname:&'static str, 
    a: i32, // p1
    b: i32, // p2
    to: &'static str // target task name
    )->i32
{
    sleep_millis!(4);
    let r = a+b;
    let ti = task_info!(qname);
    // let ti = format!("{ti} task:{tname}");
    println!("{ti}  {pad}task '{tname}' recv (#0:{a}, #1:{b}) and pass cond `a+b` [{r}] to task '{to}'.");
    r
}

pub(crate) fn ffaddx(
    qname:&'static str, pad:&'static str, tname:&'static str, 
    (a, afrom): (i32,&'static str), // p1
    (b, bfrom): (i32,&'static str), // p2
    to: &'static str // target task name
    )->i32
{
    sleep_millis!(4);
    let r = a+b;
    let ti = task_info!(qname);
    println!("{ti}  {pad}task '{tname}'recv (#0: {a} from '{afrom}', #1:{b} from '{bfrom}') and pass cond `a+b` [{r}] to task '{to}'.");
    r
}

// fun with no param and return i32
pub(crate) fn ffpr(
    qname:&'static str, pad:&'static str, tname:&'static str, 
    a: i32,
    (r, to): (i32, &'static str)
    )->i32
{
    sleep_millis!(2);
    let ti = task_info!(qname);
    println!("{ti}  {pad}task '{tname}' recv (#0: {a}) and pass cond [{r}] to task '{to}'.");
    r
}

// fun with no param and return i32
pub(crate) fn ffr(
    qname:&'static str, pad:&'static str, tname:&'static str, 
    (r, to): (i32, &'static str)
    )->i32
{
    sleep_millis!();
    let ti = task_info!(qname);
    println!("{ti}  {pad}task '{tname}' pass cond [{r}] to task '{to}'.");
    r
}

// fun with 1 param and return i32
pub(crate) fn ffp(
    qname:&'static str, pad:&'static str,tname:&'static str, 
    a: i32
    )
{
    sleep_millis!(2);
    let ti = task_info!(qname);
    println!("{ti}  {pad}task '{tname}' recv (#0: {a}).");
}


// fun with no param and No return
pub(crate) fn ff(
    qname:&'static str, pad:&'static str, tname:&'static str,
    content:&'static str
    )
{
    sleep_millis!();
    let ti = task_info!(qname);
    println!("{ti}  {pad}task '{tname}' say : '{content}'");
}

pub(crate) fn exit_ffpr(
    qname:&'static str, pad:&'static str,tname:&'static str, 
    a: i32, // p1
    to: &'static str  // return
    )->i32
{
    sleep_millis!(2);
    let ti = task_info!(qname);
    println!("{ti}  {pad}task-exit '{tname}' recv (#0: {a}) and pass cond `a` [{a}] to exit task '{to}'. and EXIT");
    a
}

pub(crate) fn exit_ff(
    qname:&'static str, pad:&'static str, tname:&'static str,
    a: i32
    )
{
    sleep_millis!();
    let ti = task_info!(qname);
    println!("{ti}  {pad}task-exit '{tname}' recv (#0: {a}) and EXIT.");
}
