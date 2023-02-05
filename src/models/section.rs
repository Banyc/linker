// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub trait SectionIndex: Copy + Clone + PartialEq + Eq + std::fmt::Debug {}

pub trait LoadableSectionTable {
    fn len(&self, index: impl SectionIndex) -> usize;
    fn address(&self, index: impl SectionIndex) -> usize;
}
