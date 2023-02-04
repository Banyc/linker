use std::{num::Wrapping, ops::Deref};

use models::*;

pub mod models;

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
