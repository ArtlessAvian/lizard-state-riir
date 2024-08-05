#[derive(Debug)]
pub struct Writer<T, Payload> {
    contents: T,
    log: Vec<Payload>,
}

impl<T, Payload> Writer<T, Payload> {
    pub fn new(contents: T) -> Self {
        Self {
            contents,
            log: vec![],
        }
    }

    pub fn new_with_log(contents: T, log: Vec<Payload>) -> Self {
        Self { contents, log }
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

    // Kind of a brute force tool.
    // See if you can use Writer::map or Writer::zip to accomplish what you're doing.
    // If not, here you go.
    pub fn split_contents(self) -> (T, Writer<(), Payload>) {
        let (contents, log) = self.into_both();
        (contents, Writer::new_with_log((), log))
    }

    // Doesn't work since `self.contents` has already been moved to make `next`. Instead we prepend_log.
    // #[must_use]
    // pub fn compose<U>(mut self, mut next: Writer<U, Payload>) -> Writer<U, Payload> {
    //     self.log.append(&mut next.log);
    //     next.log = self.log;
    //     next
    // }

    fn prepend_log(self, mut prepend: Vec<Payload>) -> Writer<T, Payload> {
        let (contents, mut log) = self.into_both();
        prepend.append(&mut log);
        Writer::new_with_log(contents, prepend)
    }

    #[must_use]
    pub fn map<U, F>(self, f: F) -> Writer<U, Payload>
    where
        F: FnOnce(T) -> U,
    {
        let (contents, log) = self.into_both();
        let next = f(contents);
        Writer::new_with_log(next, log)
    }

    #[must_use]
    pub fn bind<U, F>(self, f: F) -> Writer<U, Payload>
    where
        F: FnOnce(T) -> Writer<U, Payload>,
    {
        let (contents, log) = self.into_both();
        let next = f(contents);
        next.prepend_log(log)
    }

    #[must_use]
    pub fn bind_or_noop<F>(self, f: F) -> Writer<T, Payload>
    where
        F: FnOnce(&T) -> Option<Writer<T, Payload>>,
    {
        match f(self.get_contents()) {
            Some(writer) => writer.prepend_log(self.log),
            None => self,
        }
    }

    // Restricted functions to match F's signature. Not an ideal solution but it works.
    #[must_use]
    pub fn bind_with_side_output<U, F, SideOutput>(self, f: F) -> (Writer<U, Payload>, SideOutput)
    where
        F: FnOnce(T) -> (Writer<U, Payload>, SideOutput),
    {
        let (contents, log) = self.into_both();
        let (next, side) = f(contents);
        (next.prepend_log(log), side)
    }

    // Combines two writers. Appends the log of the second to the first.
    // See Option::zip.
    #[must_use]
    pub fn zip<U>(self, other: Writer<U, Payload>) -> Writer<(T, U), Payload> {
        self.bind(|t| other.map(|u| (t, u)))
    }

    #[must_use]
    pub fn log(mut self, line: Payload) -> Self {
        self.log.push(line);
        self
    }

    #[must_use]
    pub fn log_option(mut self, line: Option<Payload>) -> Self {
        if let Some(line) = line {
            self.log.push(line);
        }
        self
    }
}

impl<Payload> Writer<(), Payload> {
    // Avoid using with split_contents, instead try map, or try zip then bind.
    // (I don't remember writing this lmao.)
    pub fn adopt_contents<T>(self, contents: T) -> Writer<T, Payload> {
        let ((), log) = self.into_both();
        Writer::new_with_log(contents, log)
    }
}
