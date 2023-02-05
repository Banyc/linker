use std::num::Wrapping;

use thiserror::Error;

use super::{models::*, ResolvingRelocation};

pub fn relocate_reference<S>(
    reference: &ResolvingRelocation<S>,
    symbol_table: &ResolvingSymbolTable<S>,
    section_table: &impl LoadableSectionTable<S>,
) -> Result<usize, RelocationError>
where
    S: SectionIndex,
{
    // Extract helpful information
    let reference_section = reference.section;
    let SymbolValue::Defined(symbol_definition) =
        symbol_table.get(reference.symbol).value else {
            return Err(RelocationError::SymbolNotDefined);
        };
    let symbol_section = symbol_definition.section;
    let symbol_offset = symbol_definition.offset;

    // Calculate new reference value
    let new_reference_value = relocate_reference_(
        &reference,
        symbol_offset,
        section_table.address(symbol_section),
        section_table.address(reference_section),
    );

    Ok(new_reference_value)
}

#[derive(Debug, Error, PartialEq, Eq, Clone, Copy)]
pub enum RelocationError {
    #[error("Symbol not defined")]
    SymbolNotDefined,
}

fn relocate_reference_<S>(
    reference: &ResolvingRelocation<S>,
    symbol_offset: usize,
    new_symbol_section_address: usize,
    new_ref_section_address: usize,
) -> usize
where
    S: SectionIndex,
{
    let new_reference_value = match reference.typ {
        RelocationType::PcRelative => {
            let new_ref_address = reference.offset + new_ref_section_address;
            let new_symbol_address = symbol_offset + new_symbol_section_address;
            let relative_new_symbol_address =
                Wrapping(new_symbol_address) - Wrapping(new_ref_address);
            relative_new_symbol_address.0
        }
        RelocationType::Absolute => {
            let new_symbol_address = symbol_offset + new_symbol_section_address;
            new_symbol_address
        }
    };

    (new_reference_value as isize + reference.addend) as usize
}
