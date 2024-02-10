use std::marker::PhantomData;

pub trait Logger {
    type Progress<'a>: Progress<'a>
    where
        Self: 'a;

    /// Create a new **Progress** and return it
    fn create_progress(&mut self, title: String, len: usize) -> Self::Progress<'_>;
    /// Log a **info** message
    fn info(&mut self, str: impl AsRef<str>);
    /// Log a message
    fn good(&mut self, str: impl AsRef<str>);
    /// Log a **warning** message
    fn warn(&mut self, str: impl AsRef<str>);
    /// Log a **error** message
    fn error(&mut self, str: impl AsRef<str>);
}

/// The [`Progress`] trait defines methods for reporting progress
/// during long running operations. This can be implemented to provide
/// visual feedback in a UI or log output.
pub trait Progress<'a> {
    /// add one to the progress.
    fn add_progress(&mut self);
    /// end the progress.
    fn end_progress(&mut self);
}

/// Provides methods to wrap an [`ExactSizeIterator`] in a [`ProgressIter`]
/// to report progress on each iteration easier.
pub(crate) trait ProgressIterator: ExactSizeIterator + Sized {
    /// Wraps an iterator in a [`ProgressIter`] struct to report progress on each iteration to the provided [`Progress`].
    ///
    /// Begins a new progress report with the given title and length. Returns a [`ProgressIter`]
    /// that updates the progress on each iteration of the underlying iterator.
    fn progress<L: Logger, S: AsRef<str>>(
        self,
        logger: &mut L,
        title: S,
    ) -> ProgressIter<'_, L::Progress<'_>, Self> {
        let progress = logger.create_progress(title.as_ref().to_owned(), self.len());

        ProgressIter {
            progress,
            underlying: self,
            _phantom: PhantomData,
        }
    }
}

/// [`ProgressIter`] is a struct that wraps an Iterator
/// and updates a [`Progress`] on each iteration. It is used to
/// provide progress feedback when iterating over a collection.
pub(crate) struct ProgressIter<'a, P, I>
where
    P: Progress<'a>,
    I: Iterator,
{
    progress: P,
    underlying: I,
    _phantom: PhantomData<&'a P>,
}

impl<'a, P, I> Iterator for ProgressIter<'a, P, I>
where
    I: Iterator,
    P: Progress<'a>,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.underlying.next();

        match item.is_some() {
            true => self.progress.add_progress(),
            false => self.progress.end_progress(),
        }

        item
    }
}

impl<I: ExactSizeIterator> ProgressIterator for I {}
