use super::*;

pub struct InMemoryRelocatableObject {
    pub section_table: InMemoryLoadableSectionTable,
    pub symbol_table: SymbolTable<'static, InMemorySectionIndex>,
    pub references: Vec<Relocation<InMemorySectionIndex>>,
}
