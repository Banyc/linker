use thiserror::Error;

use crate::core::{
    models::ResolvingSymbolTable, relocate::relocate_reference,
    resolve::resolve_unloadable_sections, RelocationError, ResolveError,
};

use super::models::{
    object::InMemoryRelocatableObject,
    section::{InMemoryLoadableSectionTable, InMemorySectionIndex},
};

pub fn link(
    objects: Vec<InMemoryRelocatableObject>,
    address_len: usize,
) -> Result<InMemoryLoadableSectionTable, LinkError> {
    let mut section_table = InMemoryLoadableSectionTable::new();
    let mut symbol_table = ResolvingSymbolTable::new();
    let mut references = Vec::new();

    // Resolve objects
    for object in objects.into_iter() {
        // Resolve unloadable sections
        resolve_unloadable_sections(
            &mut section_table,
            &mut symbol_table,
            object.symbol_table,
            &mut references,
            object.references,
        )?;

        // Merge loadable sections
        section_table.merge(object.section_table);
    }

    // Relocate references
    for reference in references {
        // Calculate new reference value
        let new_reference_value = relocate_reference(&reference, &symbol_table, &section_table)?;

        // Update the reference value in the corresponding section
        section_table.section_mut(reference.section)
            [reference.offset..reference.offset + address_len]
            .copy_from_slice(&new_reference_value.to_le_bytes()[..address_len]);
    }

    Ok(section_table)
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum LinkError {
    #[error("Resolve error: {0}")]
    ResolveError(ResolveError<InMemorySectionIndex>),
    #[error("Relocation error: {0}")]
    RelocationError(RelocationError),
}
impl From<ResolveError<InMemorySectionIndex>> for LinkError {
    fn from(value: ResolveError<InMemorySectionIndex>) -> Self {
        Self::ResolveError(value)
    }
}
impl From<RelocationError> for LinkError {
    fn from(value: RelocationError) -> Self {
        Self::RelocationError(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::models::{
            Relocation, RelocationType, Symbol, SymbolDefinition, SymbolTable, SymbolValue,
        },
        in_memory::{
            link,
            models::{object::InMemoryRelocatableObject, section::InMemoryLoadableSectionTable},
        },
    };

    const ADDRESS_LEN: usize = 4;

    #[test]
    fn main_sum_ok() {
        let objects = vec![main_o(), sum_o()];

        // Link
        let section_table = link(objects, ADDRESS_LEN).unwrap();

        // Check result
        assert_eq!(
            section_table.sections().nth(0).unwrap(),
            &vec![
                // 0000000000000000 <main>
                0x48, 0x83, 0xec, 0x08, // sub rsp, 8
                0xbe, 0x02, 0x00, 0x00, 0x00, // mov esi, 2
                0xbf, 0x33, 0x00, 0x00, 0x00, // mov edi, array
                0xe8, 0x05, 0x00, 0x00, 0x00, // call +9 <sum>
                0x48, 0x83, 0xc4, 0x08, // add rsp, 8
                0xc3, // ret
                // 0000000000000018 <sum>
                0xb8, 0x00, 0x00, 0x00, 0x00, // mov eax, 0
                0xba, 0x00, 0x00, 0x00, 0x00, // mov edx, 0
                0xeb, 0x09, // jmp +9
                0x48, 0x63, 0xca, // movsxd rcx, edx
                0x03, 0x04, 0x8f, // add eax, [rdi + rcx * 4]
                0x83, 0xc2, 0x01, // add edx, 1
                0x39, 0xf2, // cmp edx, esi
                0x7c, 0xf3, // jl -13
                0xf3, 0xc3, // rep ret
            ]
        );
        assert_eq!(
            section_table.sections().nth(1).unwrap(),
            &vec![
                // 0000000000000033 <array>
                0x01, 0x00, 0x00, 0x00, // array: .int 1
                0x02, 0x00, 0x00, 0x00, // .int 2
            ]
        );
    }

    #[test]
    fn sum_main_ok() {
        let objects = vec![sum_o(), main_o()];

        // Link
        let section_table = link(objects, ADDRESS_LEN).unwrap();

        // Check result
        assert_eq!(
            section_table.sections().nth(0).unwrap(),
            &vec![
                // 0000000000000000 <sum>
                0xb8, 0x00, 0x00, 0x00, 0x00, // mov eax, 0
                0xba, 0x00, 0x00, 0x00, 0x00, // mov edx, 0
                0xeb, 0x09, // jmp +9
                0x48, 0x63, 0xca, // movsxd rcx, edx
                0x03, 0x04, 0x8f, // add eax, [rdi + rcx * 4]
                0x83, 0xc2, 0x01, // add edx, 1
                0x39, 0xf2, // cmp edx, esi
                0x7c, 0xf3, // jl -13
                0xf3, 0xc3, // rep ret
                // 000000000000001b <main>
                0x48, 0x83, 0xec, 0x08, // sub rsp, 8
                0xbe, 0x02, 0x00, 0x00, 0x00, // mov esi, 2
                0xbf, 0x33, 0x00, 0x00, 0x00, // mov edi, array
                0xe8, 0xd2, 0xff, 0xff, 0xff, // call -2a <sum>
                0x48, 0x83, 0xc4, 0x08, // add rsp, 8
                0xc3, // ret
            ]
        );
        assert_eq!(
            section_table.sections().nth(1).unwrap(),
            &vec![
                // 0000000000000033 <array>
                0x01, 0x00, 0x00, 0x00, // array: .int 1
                0x02, 0x00, 0x00, 0x00, // .int 2
            ]
        );
    }

    fn main_o() -> InMemoryRelocatableObject<'static> {
        let mut section_table = InMemoryLoadableSectionTable::new();
        let mut symbol_table = SymbolTable::new();
        let mut references = Vec::new();

        // Add loadable sections
        let text_section = section_table.add_section(vec![
            0x48, 0x83, 0xec, 0x08, // sub rsp, 8
            0xbe, 0x02, 0x00, 0x00, 0x00, // mov esi, 2
            0xbf, 0x00, 0x00, 0x00, 0x00, // mov edi, array
            0xe8, 0x00, 0x00, 0x00, 0x00, // call sum
            0x48, 0x83, 0xc4, 0x08, // add rsp, 8
            0xc3, // ret
        ]);
        let data_section = section_table.add_section(vec![
            0x01, 0x00, 0x00, 0x00, // array: .int 1
            0x02, 0x00, 0x00, 0x00, // .int 2
        ]);

        // Add symbols
        let sum_symbol = symbol_table.add(Symbol {
            name: "sum",
            value: SymbolValue::Undefined,
        });
        let array_symbol = symbol_table.add(Symbol {
            name: "array",
            value: SymbolValue::Defined(SymbolDefinition {
                section: data_section,
                offset: 0,
                size: 8,
            }),
        });

        // Add references
        references.push(Relocation {
            offset: 0xf,
            symbol: sum_symbol,
            typ: RelocationType::PcRelative,
            addend: -4,
            section: text_section,
        });
        references.push(Relocation {
            offset: 0xa,
            symbol: array_symbol,
            typ: RelocationType::Absolute,
            addend: 0,
            section: text_section,
        });

        InMemoryRelocatableObject {
            section_table,
            symbol_table,
            references,
        }
    }

    fn sum_o() -> InMemoryRelocatableObject<'static> {
        let mut section_table = InMemoryLoadableSectionTable::new();
        let mut symbol_table = SymbolTable::new();
        let references = Vec::new();

        // Add loadable sections
        let text_section = section_table.add_section(vec![
            0xb8, 0x00, 0x00, 0x00, 0x00, // mov eax, 0
            0xba, 0x00, 0x00, 0x00, 0x00, // mov edx, 0
            0xeb, 0x09, // jmp +9
            0x48, 0x63, 0xca, // movsxd rcx, edx
            0x03, 0x04, 0x8f, // add eax, [rdi + rcx * 4]
            0x83, 0xc2, 0x01, // add edx, 1
            0x39, 0xf2, // cmp edx, esi
            0x7c, 0xf3, // jl -13
            0xf3, 0xc3, // rep ret
        ]);
        let _data_section = section_table.add_section(vec![]);

        // Add symbols
        let _sum_symbol = symbol_table.add(Symbol {
            name: "sum",
            value: SymbolValue::Defined(SymbolDefinition {
                section: text_section,
                offset: 0,
                size: 0,
            }),
        });

        // Add references
        // None

        InMemoryRelocatableObject {
            section_table,
            symbol_table,
            references,
        }
    }
}
