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
impl InMemoryLoadableSectionTable {
    pub fn new() -> Self {
        Self { sections: vec![] }
    }
    pub fn add_section(&mut self, section: Vec<u8>) -> InMemorySectionIndex {
        let index = self.sections.len();
        self.sections.push(section);
        InMemorySectionIndex(index)
    }
    pub fn merge(&mut self, other: Self) {
        for (i, section) in other.sections.into_iter().enumerate() {
            if i < self.sections.len() {
                self.sections[i].extend(section);
            } else {
                self.sections.push(section);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InMemorySectionIndex(usize);
impl SectionIndex for InMemorySectionIndex {}
