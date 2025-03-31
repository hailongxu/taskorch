use std::{
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    sync::{atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    fmt::Debug,
};


#[derive(Clone,Copy)]
pub enum TaskKind {
    Normal,
    Exit,
}

pub struct ZERO;
pub struct ONE;
pub struct TWO;

struct FC<N,F,C> {
    f: F,
    c: C,
    n: PhantomData<N>,
}

trait P1of {
    type P;
    fn from(&self)->Option<&Self::P>;
}
trait P2of {
    type P;
    fn from(&self)->Option<&Self::P>;
}

trait Task<PC> {
    type Output;
    fn run(&self,p:PC)->Self::Output;
    fn kind(&self)->TaskKind;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct V<T>(pub T);


impl P1of for () {
    type P = ();
    fn from(&self)->Option<&Self::P> {
        None
    }
}

impl P2of for () {
    type P = ();
    fn from(&self)->Option<&Self::P> {
        None
    }
}


impl<T1> P1of for (V<T1>,) {
    type P = T1;

    fn from(&self)->Option<&Self::P> {
        Some(&self.0.0)
    }
}
impl<T1> P1of for (PhantomData<T1>,) {
    type P = T1;
    fn from(&self)->Option<&Self::P> {
        None
    }
}

impl<T1,T2> P1of for (V<T1>,T2) {
    type P = T1;

    fn from(&self)->Option<&Self::P> {
        Some(&self.0.0)
    }
}

impl<T1,T2> P1of for (PhantomData<T1>,T2) {
    type P = T1;
    fn from(&self)->Option<&Self::P> {
        None
    }
}

impl<T1,T2> P2of for (T1,V<T2>) {
    type P = T2;
    fn from(&self)->Option<&Self::P> {
        Some(&self.1.0)
    }
}
impl<T1,T2> P2of for (T1,PhantomData<T2>) {
    type P = T2;
    fn from(&self)->Option<&Self::P> {
        None
    }
}

impl<F,C,R> Task<()> for FC<ZERO,F,C>
    where
    C:CN,
    F:Fn()->R,
{
    type Output = R;
    fn run(&self,_:())->Self::Output {
        (self.f)()
    }
    fn kind(&self)->TaskKind {
        TaskKind::Normal
    }
}


impl<F,P1,C,R> Task<()> for FC<<(P1,) as CN>::N,F,C>
    where 
    F:Fn(P1)->R,
    C:P1of<P=P1>,
    P1:Clone,
{
    type Output = R;
    fn run(&self,pc:())->Self::Output {
        let p1 = if let Some(p1) = P1of::from(&self.c) {p1} else {
            todo!()
        };
        (self.f)(p1.clone())
    }
    fn kind(&self)->TaskKind {
        TaskKind::Normal
    }
}

struct ExitTask<F> {
    f:F,
}

impl<F,R> Task<()> for ExitTask<F>
    where
    F:Fn()->R,
{
    type Output = R;
    fn run(&self,_:())->Self::Output {
        (self.f)()
    }
    fn kind(&self)->TaskKind {
        TaskKind::Exit
    }
}

pub trait CN {
    type N;
}
impl CN for () {
    type N = ZERO;
}
impl<T1> CN for (T1,) {
    type N = ONE;
}
impl<T1,T2> CN for (T1,T2,) {
    type N = TWO;
}

pub fn fc<F,C:CN>(f:F,c:C)->FC<C::N,F,C> {
    FC {f,c,n:PhantomData}
}

fn fc_exit<F>(f:F)->ExitTask<F> {
    ExitTask {f}
}


#[derive(Clone)]
pub struct Queue<PC>(Arc<(Mutex<VecDeque<Box<dyn Task<PC,Output=()>+Send>>>,Condvar)>);

impl Queue<()> {
    pub fn new()->Queue<()> {
        Queue(Arc::new((Mutex::new(VecDeque::new()),Condvar::new())))
    }

    pub fn add<F,C:CN>(&mut self,f:F, c:C)
        where
        F:Send+'static,
        C:Send+'static,
        <C as CN>::N:Send+'static,
        FC<<C as CN>::N, F, C>: Task<(),Output=()>
    {
        let fc = fc(f,c);
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back(Box::new(fc));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    pub fn add_exit(&mut self, f:impl Fn()+'static+Send) {
        let exit_task = fc_exit(f);
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back(Box::new(exit_task));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    pub fn pop(&mut self)->Option<Box<dyn Task<(),Output = ()>+Send>> {
        self
            .0
            .0
            .lock()
            .unwrap()
            .pop_front()
    }
    
    pub fn clear(&mut self) {
        self
            .0
            .0
            .lock()
            .unwrap()
            .clear()
    }

    pub fn len(&self)->usize {
        self
            .0
            .0
            .lock()
            .unwrap()
            .len()
    }
}

pub fn spawn_thread(queue:&Queue<()>)-> Jhandle {
    let quit_flag = Arc::<AtomicBool>::new(AtomicBool::new(false));
    let quit = quit_flag.clone();
    let queue = queue.0.clone();
    let handle = thread::spawn(move||{
        loop {
            if quit.load(Ordering::Relaxed) {
                break;
            }
            
            let mut m = queue.0.lock().unwrap();
            let task = m.pop_front();
            if let Some(task) = task {
                drop(m);
                task.run(());
                if let TaskKind::Exit = task.kind() {
                    break;
                }
            } else {
                let _unused = queue.1.wait(m);
            }
        }
    });
    Jhandle(handle,quit_flag)
}

pub struct Jhandle(JoinHandle<()>,Arc<AtomicBool>);

impl Jhandle {
    pub fn collect_into(self, pool:&mut Pool)->Option<usize> {
        pool.insert_thread(self)
    }

    pub fn exit_next(&mut self) {
        let _ = self.1.compare_exchange(
            false, true,
            Ordering::Acquire,Ordering::Relaxed);
    }

    pub fn wait(self)->thread::Result<()> {
        self.0.join()
    }
}

pub struct Pool<PC=()> {
    queues: HashMap<usize,Queue<PC>>,
    jhands: HashMap<usize,Jhandle>,
    id_next: usize,
}

impl Pool {
    pub fn new()-> Self {
        Self {
            queues: HashMap::new(),
            jhands: HashMap::new(),
            id_next: 0,
        }
    }

    fn next_id(&mut self)->usize {
        self.id_next += 1;
        self.id_next
    }

    fn queue(&self, qid:usize)->Option<&Queue<()>> {
        self.queues.get(&qid)
    }

    fn jhandle(&self, tid:usize)->Option<&Jhandle> {
        self.jhands.get(&tid)
    }

    pub fn insert_queue(&mut self,queue:&Queue<()>)->Option<usize> {
        let id = self.next_id();
        self.queues.insert(id, queue.clone())
            .map(|_|id)
    }

    fn insert_thread_from(&mut self, qid:usize)->Option<usize> {
        let queue = self.queue(qid)?;
        spawn_thread(queue).collect_into(self)
    }

    fn insert_thread(&mut self, jhandle:Jhandle)->Option<usize> {
        let id = self.next_id();
        self.jhands.insert(id, jhandle)
            .map(|_|id)
    }

    fn exit_next(&mut self, tid:usize)->Option<()> {
        let jhandle = self.jhands.get_mut(&tid)?;
        jhandle.exit_next();
        Some(())
    }

    fn exit_next_all(&mut self) {
        for jhand in self.jhands.values_mut() {
            jhand.exit_next();
        }
    }

    pub fn wait(self) {
        for handle in self.jhands {
            handle.1.0.join().unwrap();
        }
    }
}
