use std::{collections::HashMap, ops::Deref};

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

pub struct ResolvingSymbolTable<'name, S>
where
    S: SectionIndex,
{
    inner: SymbolTable<'name, S>,
    indices: HashMap<&'name str, SymbolIndex>,
}
impl<'name, S> Deref for ResolvingSymbolTable<'name, S>
where
    S: SectionIndex,
{
    type Target = SymbolTable<'name, S>;
    fn deref(&self) -> &SymbolTable<'name, S> {
        &self.inner
    }
}
impl<'name, S> ResolvingSymbolTable<'name, S>
where
    S: SectionIndex,
{
    pub fn new() -> Self {
        Self {
            inner: SymbolTable::new(),
            indices: HashMap::new(),
        }
    }

    pub fn add(&mut self, symbol: Symbol<'name, S>) -> SymbolIndex {
        let index = self.inner.add(symbol);
        self.indices.insert(self.inner.get(index).name, index);
        index
    }

    pub fn get(&self, index: SymbolIndex) -> &Symbol<'name, S> {
        self.inner.get(index)
    }

    pub fn get_by_name(&self, name: &'name str) -> Option<&Symbol<'name, S>> {
        self.indices.get(&name).map(|index| self.inner.get(*index))
    }

    pub fn get_index_by_name(&self, name: &'name str) -> Option<SymbolIndex> {
        self.indices.get(&name).copied()
    }

    pub fn replace(&mut self, index: SymbolIndex, symbol: Symbol<'name, S>) {
        let old_symbol = self.inner.get(index);
        self.indices.remove(old_symbol.name);
        self.indices.insert(symbol.name, index);
        self.inner.replace(index, symbol);
    }
}
