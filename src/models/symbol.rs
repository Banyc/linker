use super::SectionIndex;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Symbol<'name, S>
where
    S: SectionIndex,
{
    pub name: &'name str,
    pub value: SymbolValue<S>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    pub fn replace(&mut self, index: SymbolIndex, symbol: Symbol<'name, S>) {
        self.0[index.0] = symbol;
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn into_iter(self) -> impl Iterator<Item = (SymbolIndex, Symbol<'name, S>)> {
        self.0
            .into_iter()
            .enumerate()
            .map(|(index, symbol)| (SymbolIndex(index), symbol))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymbolValue<S>
where
    S: SectionIndex,
{
    Undefined,
    Defined(SymbolDefinition<S>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SymbolDefinition<S>
where
    S: SectionIndex,
{
    pub section: S,
    pub offset: usize,
    pub size: usize,
}
