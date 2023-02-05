use std::num::Wrapping;

use super::{models::*, ResolvingRelocation, ResolvingSymbolTable};

/// # Panic
///
/// Panics if some symbols in `symbol_table` are not defined
pub fn relocate_reference<S>(
    reference: &ResolvingRelocation<S>,
    symbols: &ResolvingSymbolTable<S>,
    new_symbol_section_address: usize,
    new_ref_section_address: usize,
) -> usize
where
    S: SectionIndex,
{
    let symbol = symbols.get(reference.symbol);
    let SymbolValue::Defined(definition) = symbol.value else {
        panic!("Symbol is not defined");
    };
    let offset = definition.offset;

    let new_reference_value = match reference.typ {
        RelocationType::PcRelative => {
            let new_ref_address = reference.offset + new_ref_section_address;
            let new_symbol_address = offset + new_symbol_section_address;
            let relative_new_symbol_address =
                Wrapping(new_symbol_address) - Wrapping(new_ref_address);
            relative_new_symbol_address.0
        }
        RelocationType::Absolute => {
            let new_symbol_address = offset + new_symbol_section_address;
            new_symbol_address
        }
    };

    (new_reference_value as isize + reference.addend) as usize
}
