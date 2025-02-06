use std::ops::Deref;
use std::ops::DerefMut;

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

    pub(crate) fn new_with_log(contents: T, log: Vec<Payload>) -> Self {
        Self { contents, log }
    }

    pub(crate) fn transpose(option: Option<Self>) -> Writer<Option<T>, Payload> {
        option.map_or(Writer::new(None), |some| some.map(Some))
    }

    pub(crate) fn get_contents(&self) -> &T {
        &self.contents
    }

    #[allow(dead_code)]
    pub(crate) fn get_log(&self) -> &Vec<Payload> {
        &self.log
    }

    pub fn into_both(self) -> (T, Vec<Payload>) {
        (self.contents, self.log)
    }

    pub(crate) fn map<U, F>(self, f: F) -> Writer<U, Payload>
    where
        F: FnOnce(T) -> U,
    {
        Writer::<U, Payload> {
            contents: f(self.contents),
            log: self.log,
        }
    }

    pub fn bind<U, F>(self, f: F) -> Writer<U, Payload>
    where
        F: FnOnce(T) -> Writer<U, Payload>,
    {
        self.map(f).flatten()
    }

    pub(crate) fn bind_or_noop<F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Option<Self>,
    {
        self.bind(|t| Writer::transpose(f(&t)).map(|opt| opt.unwrap_or(t)))
    }

    // /// An abomination. It *looks* clean, but on the caller side, uhhh.
    // /// I would recommend *not* creating an iterator of fs.
    // pub(crate) fn bind_compose<F, I>(self, fs: I) -> Self
    // where
    //     F: FnOnce(T) -> Self,
    //     I: IntoIterator<Item = F>,
    // {
    //     let mut out = self;
    //     for f in fs {
    //         out = out.bind(f);
    //     }
    //     out
    // }

    #[expect(dead_code)]
    pub(crate) fn borrow_and_pair<U, F>(self, f: F) -> Writer<(T, U), Payload>
    where
        F: FnOnce(&T) -> U,
    {
        let apply = f(&self.contents);
        self.make_pair(apply)
    }

    #[expect(dead_code)]
    pub(crate) fn borrow_and_zip<U, F>(self, f: F) -> Writer<(T, U), Payload>
    where
        F: FnOnce(&T) -> Writer<U, Payload>,
    {
        let to_zip = f(self.get_contents());
        self.zip(to_zip)
    }

    // Combines two writers. Appends the log of the second to the first.
    // See Option::zip.
    pub(crate) fn zip<U>(self, other: Writer<U, Payload>) -> Writer<(T, U), Payload> {
        self.bind(|t| other.map(|u| (t, u)))
    }

    // Appends the log of the second to the first.
    // See Option::zip.
    pub(crate) fn zip_nothing(self, other: Writer<(), Payload>) -> Writer<T, Payload> {
        self.bind(|t| other.take(t))
    }

    pub(crate) fn log(mut self, line: Payload) -> Self {
        self.log.push(line);
        self
    }

    #[expect(dead_code)]
    pub(crate) fn peek_and_log<F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Payload,
    {
        let line = f(&self.contents);
        self.log(line)
    }

    pub fn log_each(mut self, lines: impl IntoIterator<Item = Payload>) -> Self {
        self.log.extend(lines);
        self
    }

    pub(crate) fn log_option(self, line: Option<Payload>) -> Self {
        self.log_each(line)
    }

    pub(crate) fn make_pair<U>(self, value: U) -> Writer<(T, U), Payload> {
        self.map(|x| (x, value))
    }
}

impl<T, Payload> Writer<Writer<T, Payload>, Payload> {
    pub(crate) fn flatten(self) -> Writer<T, Payload> {
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

    #[expect(dead_code)]
    pub(crate) fn swap_pair(self) -> Writer<(Extract, Keep), Payload> {
        self.map(|(t, u)| (u, t))
    }
}

/// There are some manips you can do with this.
/// If you can, prefer `Vec<Payload>` or `impl IntoIterator<Item = Payload>`.
impl<Payload> Writer<(), Payload> {
    #[expect(dead_code)]
    pub(crate) fn new_payload(payload: Payload) -> Self {
        Self {
            contents: (),
            log: vec![payload],
        }
    }

    pub(crate) fn from_payloads(payloads: impl IntoIterator<Item = Payload>) -> Self {
        Self {
            contents: (),
            log: payloads.into_iter().collect(),
        }
    }

    /// More literate version of `map(|()| t)`
    // Do not implement for Writer<T>, since that would mean T gets dropped!
    pub(crate) fn take<T>(self, t: T) -> Writer<T, Payload> {
        self.map(|()| t)
    }

    /// Prepend log.
    pub(crate) fn take_writer<T>(self, other: Writer<T, Payload>) -> Writer<T, Payload> {
        self.take(other).flatten()
    }

    #[expect(unused)]
    pub(crate) fn into_log(self) -> Vec<Payload> {
        self.log
    }
}

impl<T, Payload> Writer<&mut T, Payload> {
    /// More literate version of `map(drop)`
    // Do not implement for Writer<T>, since that would mean T wasn't meaningfully used! (or was used as ref.)
    #[expect(dead_code)]
    pub(crate) fn drop_ref(self) -> Writer<(), Payload> {
        self.map(drop)
    }
}

impl<T, Payload> Writer<&T, Payload> {
    /// More literate version of `map(drop)`
    // Do not implement for Writer<T>, since that would mean T wasn't meaningfully used! (or was used as ref.)
    #[expect(dead_code)]
    pub(crate) fn drop_ref(self) -> Writer<(), Payload> {
        self.map(drop)
    }
}

impl<T, Payload> Deref for Writer<T, Payload> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<T, Payload> DerefMut for Writer<T, Payload> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contents
    }
}

impl<Payload> FromIterator<Payload> for Writer<(), Payload> {
    fn from_iter<T: IntoIterator<Item = Payload>>(iter: T) -> Self {
        Self::from_payloads(iter)
    }
}
