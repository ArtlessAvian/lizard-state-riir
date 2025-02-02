#[derive(Debug)]
#[must_use]
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

    pub fn transpose(option: Option<Self>) -> Writer<Option<T>, Payload> {
        option.map_or(Writer::new(None), |some| some.map(Some))
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

    pub fn map<U, F>(self, f: F) -> Writer<U, Payload>
    where
        F: FnOnce(T) -> U,
    {
        let Writer { contents, log } = self;
        Writer::new_with_log(f(contents), log)
    }

    pub fn bind<U, F>(self, f: F) -> Writer<U, Payload>
    where
        F: FnOnce(T) -> Writer<U, Payload>,
    {
        self.map(f).flatten()
    }

    pub fn bind_or_noop<F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Option<Self>,
    {
        self.bind(|t| Writer::transpose(f(&t)).map(|opt| opt.unwrap_or(t)))
    }

    pub fn borrow_and_pair<U, F>(self, f: F) -> Writer<(T, U), Payload>
    where
        F: FnOnce(&T) -> U,
    {
        let apply = f(&self.contents);
        self.make_pair(apply)
    }

    pub fn borrow_and_zip<U, F>(self, f: F) -> Writer<(T, U), Payload>
    where
        F: FnOnce(&T) -> Writer<U, Payload>,
    {
        let to_zip = f(self.get_contents());
        self.zip(to_zip)
    }

    // Combines two writers. Appends the log of the second to the first.
    // See Option::zip.
    pub fn zip<U>(self, other: Writer<U, Payload>) -> Writer<(T, U), Payload> {
        self.bind(|t| other.map(|u| (t, u)))
    }

    pub fn log(mut self, line: Payload) -> Self {
        self.log.push(line);
        self
    }

    pub fn log_each(mut self, lines: impl IntoIterator<Item = Payload>) -> Self {
        self.log.extend(lines);
        self
    }

    pub fn log_option(self, line: Option<Payload>) -> Self {
        self.log_each(line)
    }

    pub fn make_pair<U>(self, value: U) -> Writer<(T, U), Payload> {
        self.map(|x| (x, value))
    }
}

impl<T, Payload> Writer<Writer<T, Payload>, Payload> {
    fn flatten(self) -> Writer<T, Payload> {
        let Writer {
            contents: inner,
            log: mut outer_log,
        } = self;
        let Writer {
            contents,
            log: mut inner_log,
        } = inner;
        outer_log.append(&mut inner_log);
        Writer::new_with_log(contents, outer_log)
    }
}

impl<Keep, Extract, Payload> Writer<(Keep, Extract), Payload> {
    /// Can be thought of as transposing Pair and Writer.
    /// We arbitrarily choose the first of the pair to be kept.
    /// We can also swap the pair if needed.
    pub fn split_pair(self) -> (Writer<Keep, Payload>, Extract) {
        let Writer {
            contents: (keep, extract),
            log,
        } = self;
        (Writer::new_with_log(keep, log), extract)
    }

    pub fn swap_pair(self) -> Writer<(Extract, Keep), Payload> {
        self.map(|(t, u)| (u, t))
    }
}
