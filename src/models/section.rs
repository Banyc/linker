// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub trait SectionIndex: Copy + Clone + PartialEq + Eq + std::fmt::Debug {}

pub trait SectionTable {
    fn new() -> Self;
    fn len(&self, index: impl SectionIndex) -> usize;
    fn address(&self, index: impl SectionIndex) -> usize;
    fn merge(&mut self, other: Self);
}
