// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub trait SectionIndex: Copy + Clone + PartialEq + Eq + std::fmt::Debug {}

pub trait LoadableSectionTable<S>
where
    S: SectionIndex,
{
    fn len(&self, index: S) -> usize;
    fn address(&self, index: S) -> usize;
}
