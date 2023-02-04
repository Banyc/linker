use std::{collections::HashMap, num::Wrapping, ops::Deref};

use models::*;

pub mod models;

pub fn resolve<'name, S, ST>(
    section_table: &mut ST,
    other_section_table: ST,
    symbol_table: &mut ResolvedSymbolTable<'name, S>,
    other_symbol_table: SymbolTable<'name, S>,
    relocation_table: &mut Vec<ResolvedRelocation<S>>,
    other_relocation_table: Vec<Relocation<S>>,
) -> Result<(), ResolveError>
where
    S: SectionIndex,
    ST: LoadableSectionTable,
{
    let mut resolved_symbols = HashMap::with_capacity(other_symbol_table.len());

    // Resolve relocations
    for reference in other_relocation_table.into_iter() {
        let symbol = other_symbol_table.get(reference.symbol);

        // Update symbol offset and store the symbol
        let new_symbol = match resolved_symbols.get(&reference.symbol) {
            Some(symbol) => *symbol,
            None => resolve_symbol(section_table, symbol_table, symbol)?,
        };

        // Update relocation offset and symbol index
        let new_reference = ResolvedRelocation(Relocation {
            symbol: new_symbol,
            offset: section_table.len(reference.section) + reference.offset,
            ..reference
        });

        // Store resolved relocation
        relocation_table.push(new_reference);

        // Mark the symbol as resolved
        resolved_symbols.insert(reference.symbol, new_symbol);
    }

    // Resolve symbols
    for (index, symbol) in other_symbol_table.into_iter() {
        // Skip if already resolved
        if resolved_symbols.contains_key(&index) {
            continue;
        }

        // Update offset
        resolve_symbol(section_table, symbol_table, &symbol)?;
    }

    // Merge section tables
    section_table.merge(other_section_table);

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResolveError {
    ConflictSymbols,
}

fn resolve_symbol<'name, S>(
    section_table: &mut impl LoadableSectionTable,
    symbol_table: &mut ResolvedSymbolTable<'name, S>,
    symbol: &Symbol<'name, S>,
) -> Result<SymbolIndex, ResolveError>
where
    S: SectionIndex,
{
    let existing_symbol_index = symbol_table.get_index_by_name(symbol.name);

    let new_symbol_index = match existing_symbol_index {
        Some(existing_symbol_index) => {
            let existing_symbol = symbol_table.get(existing_symbol_index);
            let section = match (existing_symbol.section, symbol.section) {
                (SymbolSection::Undefined, SymbolSection::Undefined) => {
                    return Ok(existing_symbol_index)
                }
                (SymbolSection::Undefined, SymbolSection::Defined(section)) => section,
                (SymbolSection::Defined(_), SymbolSection::Undefined) => {
                    return Ok(existing_symbol_index)
                }
                (SymbolSection::Defined(_), SymbolSection::Defined(_)) => {
                    return Err(ResolveError::ConflictSymbols)
                }
            };

            // Replace the existing symbol
            let new_symbol = Symbol {
                offset: section_table.len(section) + symbol.offset,
                ..*symbol
            };
            symbol_table.replace(existing_symbol_index, new_symbol);
            existing_symbol_index
        }
        None => {
            // Add the symbol to symbol table
            let new_symbol = match symbol.section {
                SymbolSection::Undefined => *symbol,
                SymbolSection::Defined(section) => Symbol {
                    offset: section_table.len(section) + symbol.offset,
                    ..*symbol
                },
            };
            let new_symbol_index = symbol_table.add(new_symbol);
            new_symbol_index
        }
    };

    return Ok(new_symbol_index);
}

pub fn relocate<S>(
    reference: &ResolvedRelocation<S>,
    symbols: &ResolvedSymbolTable<S>,
    new_symbol_section_address: usize,
    new_ref_section_address: usize,
) -> usize
where
    S: SectionIndex,
{
    let symbol = symbols.get(reference.symbol);

    let new_reference_value = match reference.typ {
        RelocationType::PcRelative => {
            let new_ref_address = reference.offset + new_ref_section_address;
            let new_symbol_address = symbol.offset + new_symbol_section_address;
            let relative_new_symbol_address =
                Wrapping(new_symbol_address) - Wrapping(new_ref_address);
            relative_new_symbol_address.0
        }
        RelocationType::Absolute => {
            let new_symbol_address = symbol.offset + new_symbol_section_address;
            new_symbol_address
        }
    };

    (new_reference_value as isize + reference.addend) as usize
}

pub struct ResolvedRelocation<S>(Relocation<S>)
where
    S: SectionIndex;
impl<S> Deref for ResolvedRelocation<S>
where
    S: SectionIndex,
{
    type Target = Relocation<S>;
    fn deref(&self) -> &Relocation<S> {
        &self.0
    }
}

pub struct ResolvedSymbolTable<'name, S>
where
    S: SectionIndex,
{
    inner: SymbolTable<'name, S>,
    indices: HashMap<&'name str, SymbolIndex>,
}
impl<'name, S> Deref for ResolvedSymbolTable<'name, S>
where
    S: SectionIndex,
{
    type Target = SymbolTable<'name, S>;
    fn deref(&self) -> &SymbolTable<'name, S> {
        &self.inner
    }
}
impl<'name, S> ResolvedSymbolTable<'name, S>
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
        self.inner.replace(index, symbol);
    }
}
