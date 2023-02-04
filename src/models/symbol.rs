use super::SectionIndex;

pub struct Symbol<'name, S>
where
    S: SectionIndex,
{
    pub name: &'name str,
    pub section: S,
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SymbolIndex(usize);

pub struct SymbolTable<'name, S>(Vec<Symbol<'name, S>>)
where
    S: SectionIndex;
impl<'name, S> SymbolTable<'name, S>
where
    S: SectionIndex,
{
    pub fn new() -> Self {
        SymbolTable(Vec::new())
    }
    pub fn add(&mut self, symbol: Symbol<'name, S>) -> SymbolIndex {
        let index = SymbolIndex(self.0.len());
        self.0.push(symbol);
        index
    }
    pub fn get(&self, index: SymbolIndex) -> &Symbol<'name, S> {
        &self.0[index.0]
    }
}
