use std::{collections::HashSet, num::Wrapping, ops::Deref};

use models::*;

pub mod models;

pub fn resolve<'name, S, ST>(
    section_table: &mut ST,
    new_section_table: ST,
    symbol_table: &mut ResolvedSymbolTable<'name, S>,
    new_symbol_table: SymbolTable<'name, S>,
    relocation_table: &mut Vec<ResolvedRelocation>,
    new_relocation_table: Vec<Relocation>,
) where
    S: SectionIndex,
    ST: LoadableSectionTable,
{
    let mut resolved_symbols = HashSet::with_capacity(new_symbol_table.len());

    // Resolve relocations
    for reference in new_relocation_table.into_iter() {
        let symbol = symbol_table.get(reference.symbol);

        // Update symbol offset and store the symbol
        let new_symbol = Symbol {
            offset: section_table.len(symbol.section) + symbol.offset,
            ..*symbol
        };
        let new_symbol_index = symbol_table.0.add(new_symbol);

        // Update relocation offset
        let new_reference = ResolvedRelocation(Relocation {
            symbol: new_symbol_index,
            ..reference
        });

        // Store resolved relocation
        relocation_table.push(new_reference);

        // Mark the symbol as resolved
        resolved_symbols.insert(reference.symbol);
    }

    // Resolve symbols
    for (index, symbol) in new_symbol_table.into_iter() {
        // Skip if already resolved
        if resolved_symbols.contains(&index) {
            continue;
        }

        // Update offset
        let new_symbol = Symbol {
            offset: section_table.len(symbol.section) + symbol.offset,
            ..symbol
        };
        symbol_table.0.add(new_symbol);
    }

    // Merge section tables
    section_table.merge(new_section_table);
}

pub fn relocate<S>(
    reference: &ResolvedRelocation,
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

pub struct ResolvedRelocation(Relocation);
impl Deref for ResolvedRelocation {
    type Target = Relocation;
    fn deref(&self) -> &Relocation {
        &self.0
    }
}

pub struct ResolvedSymbolTable<'name, S>(SymbolTable<'name, S>)
where
    S: SectionIndex;
impl<'name, S> Deref for ResolvedSymbolTable<'name, S>
where
    S: SectionIndex,
{
    type Target = SymbolTable<'name, S>;
    fn deref(&self) -> &SymbolTable<'name, S> {
        &self.0
    }
}
