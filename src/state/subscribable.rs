pub struct Subscribable<A> {
    val: Option<A>,
}

impl<A> Subscribable<A> {
    pub fn new() -> Self {
        Self { val: None }
    }
}

impl<A: Clone> Subscribable<A> {
    pub fn get_val(&self) -> Option<A> {
        self.val.clone()
    }
}
