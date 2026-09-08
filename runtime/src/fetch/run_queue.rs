use super::continuation::Job;

#[derive_where::derive_where(Default)]
pub(super) struct RunQueue<E>(Vec<Job<E>>);

impl<E> RunQueue<E> {
    pub(super) fn prepend(
        &mut self,
        jobs: impl IntoIterator<Item = Job<E>, IntoIter: DoubleEndedIterator>,
    ) {
        self.0.extend(jobs.into_iter().rev());
    }

    pub(super) fn next(&mut self) -> Option<Job<E>> {
        self.0.pop()
    }
}
