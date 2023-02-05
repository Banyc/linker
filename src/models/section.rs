// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub trait SectionIndex: Copy + Clone + PartialEq + Eq + std::fmt::Debug {}

pub trait LoadableSectionTable<S>
where
    S: SectionIndex,
{
    fn len(&self, index: S) -> usize;
    fn address(&self, index: S) -> usize;
}

pub struct InMemoryLoadableSectionTable {
    sections: Vec<Vec<u8>>,
}
impl LoadableSectionTable<InMemorySectionIndex> for InMemoryLoadableSectionTable {
    fn len(&self, index: InMemorySectionIndex) -> usize {
        self.sections[index.0].len()
    }
    fn address(&self, index: InMemorySectionIndex) -> usize {
        let mut sum = 0;
        for i in 0..index.0 {
            sum += self.sections[i].len();
        }
        sum
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InMemorySectionIndex(usize);
impl SectionIndex for InMemorySectionIndex {}
