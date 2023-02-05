use crate::core::models::*;

pub struct InMemoryLoadableSectionTable {
    sections: Vec<Vec<u8>>,
}
impl LoadableSectionTable<InMemorySectionIndex> for InMemoryLoadableSectionTable {
    fn len(&self, index: InMemorySectionIndex) -> usize {
        match self.sections.get(index.0) {
            Some(section) => section.len(),
            None => 0,
        }
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
    pub fn sections(&self) -> impl Iterator<Item = &Vec<u8>> {
        self.sections.iter()
    }
    pub fn section_mut(&mut self, index: InMemorySectionIndex) -> &mut Vec<u8> {
        &mut self.sections[index.0]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InMemorySectionIndex(usize);
impl SectionIndex for InMemorySectionIndex {}
