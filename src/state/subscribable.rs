pub struct Subscribable<A> {
    val: Option<A>,
    callbacks: Vec<Box<dyn FnOnce(A)>>,
}

impl<A> Subscribable<A> {
    pub fn new() -> Self {
        Self {
            val: None,
            callbacks: vec![],
        }
    }

    pub fn subscribe(&mut self, callback: impl FnOnce(A) + 'static) {
        self.callbacks.push(Box::new(callback));
    }
}

impl<A: Clone> Subscribable<A> {
    pub fn get_val(&self) -> Option<A> {
        self.val.clone()
    }
}
