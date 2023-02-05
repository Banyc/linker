use super::*;

pub struct InMemoryRelocatableObject<'name> {
    pub section_table: InMemoryLoadableSectionTable,
    pub symbol_table: SymbolTable<'name, InMemorySectionIndex>,
    pub references: Vec<Relocation<InMemorySectionIndex>>,
}
