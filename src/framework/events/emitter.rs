
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotToken(usize);

pub struct EventEmitter<T> {
    senders: HashMap<SlotToken, Sender<T>>,
    args: std::marker::PhantomData<T>,
}

impl<T> EventEmitter<T> {
    pub fn new() -> Self {
        EventEmitter {
            senders: HashMap::new(),
            args: std::marker::PhantomData,
        }
    }
}

impl<T: Clone> EventEmitter<T> {
    pub fn connect(&mut self, sender: Sender<T>) -> SlotToken {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = SlotToken(COUNTER.fetch_add(1, Ordering::Relaxed));
        self.senders.insert(id, sender);
        id
    }

    pub fn disconnect(&mut self, id: &SlotToken) {
        self.senders.remove(id);
    }

    pub fn new_receiver(&mut self) -> Receiver<T> {
        let (sender, receiver) = channel();
        self.connect(sender);
        receiver
    }

    pub fn emit(&mut self, args: T) {
        self.senders
            .retain(|_, sender| sender.send(args.clone()).is_ok());
    }
}