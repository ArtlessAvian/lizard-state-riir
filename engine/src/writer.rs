pub struct Writer<T, Payload> {
    contents: T,
    log: Vec<Payload>,
}

impl<T, Payload> Writer<T, Payload> {
    pub fn new(content: T) -> Self {
        Self {
            contents: content,
            log: vec![],
        }
    }

    pub fn get_contents(&self) -> &T {
        &self.contents
    }

    pub fn get_log(&self) -> &Vec<Payload> {
        &self.log
    }

    pub fn into_both(self) -> (T, Vec<Payload>) {
        (self.contents, self.log)
    }

    #[must_use]
    pub fn compose<U>(mut self, mut next: Writer<U, Payload>) -> Writer<U, Payload> {
        self.log.append(&mut next.log);
        next.log = self.log;
        next
    }

    #[must_use]
    pub fn bind<U, F>(self, f: F) -> Writer<U, Payload>
    where
        F: FnOnce(&T) -> Writer<U, Payload>,
    {
        let next = f(&self.contents);
        self.compose(next)
    }

    // Restricted functions to match F's signature. Not an ideal solution but it works.
    #[must_use]
    pub fn bind_with_side_output<U, F, SideOutput>(self, f: F) -> (Writer<U, Payload>, SideOutput)
    where
        F: FnOnce(&T) -> (Writer<U, Payload>, SideOutput),
    {
        let (next, side) = f(&self.contents);
        (self.compose(next), side)
    }

    #[must_use]
    pub fn log(mut self, line: Payload) -> Self {
        self.log.push(line);
        self
    }
}
