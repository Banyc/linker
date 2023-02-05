use std::{collections::HashMap, num::Wrapping, ops::Deref};

use models::*;
use thiserror::Error;

pub mod models;

pub fn resolve_unloadable_sections<'name, S, ST>(
    section_table: &ST,
    symbol_table: &mut ResolvedSymbolTable<'name, S>,
    other_symbol_table: SymbolTable<'name, S>,
    relocation_table: &mut Vec<ResolvedRelocation<S>>,
    other_relocation_table: Vec<Relocation<S>>,
) -> Result<(), ResolveError>
where
    S: SectionIndex,
    ST: LoadableSectionTable<S>,
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

    Ok(())
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    #[error("conflict `{symbol}` symbols")]
    ConflictSymbols { symbol: String },
}

fn resolve_symbol<'name, S>(
    section_table: &impl LoadableSectionTable<S>,
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
            let _ = match (existing_symbol.value, symbol.value) {
                (SymbolValue::Undefined, SymbolValue::Undefined) => {
                    return Ok(existing_symbol_index)
                }
                (SymbolValue::Undefined, SymbolValue::Defined(definition)) => definition,
                (SymbolValue::Defined(_), SymbolValue::Undefined) => {
                    return Ok(existing_symbol_index)
                }
                (SymbolValue::Defined(_), SymbolValue::Defined(_)) => {
                    return Err(ResolveError::ConflictSymbols {
                        symbol: symbol.name.to_string(),
                    })
                }
            };

            // Replace the existing symbol
            let new_symbol = update_offset(section_table, symbol);
            symbol_table.replace(existing_symbol_index, new_symbol);
            existing_symbol_index
        }
        None => {
            // Add the symbol to symbol table
            let new_symbol = update_offset(section_table, symbol);
            let new_symbol_index = symbol_table.add(new_symbol);
            new_symbol_index
        }
    };

    return Ok(new_symbol_index);
}

fn update_offset<'name, S>(
    section_table: &impl LoadableSectionTable<S>,
    symbol: &Symbol<'name, S>,
) -> Symbol<'name, S>
where
    S: SectionIndex,
{
    match symbol.value {
        SymbolValue::Undefined => *symbol,
        SymbolValue::Defined(definition) => {
            let new_offset = section_table.len(definition.section) + definition.offset;
            Symbol {
                value: SymbolValue::Defined(SymbolDefinition {
                    offset: new_offset,
                    ..definition
                }),
                ..*symbol
            }
        }
    }
}

/// # Panic
///
/// Panics if some symbols in `symbol_table` are not defined
pub fn relocate_reference<S>(
    reference: &ResolvedRelocation<S>,
    symbols: &ResolvedSymbolTable<S>,
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
        let old_symbol = self.inner.get(index);
        self.indices.remove(old_symbol.name);
        self.indices.insert(symbol.name, index);
        self.inner.replace(index, symbol);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDRESS_LEN: usize = 4;

    #[test]
    fn ok() {
        let mut section_table = InMemoryLoadableSectionTable::new();
        let mut symbol_table = ResolvedSymbolTable::new();
        let mut references = Vec::new();

        // Resolve main.o
        {
            // Resolve unloadable sections
            let (other_section_table, other_symbol_table, other_references) = main_o();
            let res = resolve_unloadable_sections(
                &mut section_table,
                &mut symbol_table,
                other_symbol_table,
                &mut references,
                other_references,
            );
            assert!(res.is_ok());

            // Merge unloadable sections
            section_table.merge(other_section_table);
        }

        // Resolve sum.o
        {
            // Resolve unloadable sections
            let (other_section_table, other_symbol_table, other_references) = sum_o();
            let res = resolve_unloadable_sections(
                &mut section_table,
                &mut symbol_table,
                other_symbol_table,
                &mut references,
                other_references,
            );
            assert!(res.is_ok());

            // Merge unloadable sections
            section_table.merge(other_section_table);
        }

        // Relocate references
        for reference in references {
            let reference_section = reference.section;
            let SymbolValue::Defined(symbol_definition) = symbol_table.get(reference.symbol).value else {
                panic!("Symbol is not defined");
            };
            let symbol_section = symbol_definition.section;
            let new_reference_value = relocate_reference(
                &reference,
                &symbol_table,
                section_table.address(symbol_section),
                section_table.address(reference_section),
            );
            section_table.section_mut(reference.section)
                [reference.offset..reference.offset + ADDRESS_LEN]
                .copy_from_slice(&new_reference_value.to_le_bytes()[..ADDRESS_LEN]);
        }

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

    fn main_o() -> (
        InMemoryLoadableSectionTable,
        SymbolTable<'static, InMemorySectionIndex>,
        Vec<Relocation<InMemorySectionIndex>>,
    ) {
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

        (section_table, symbol_table, references)
    }

    fn sum_o() -> (
        InMemoryLoadableSectionTable,
        SymbolTable<'static, InMemorySectionIndex>,
        Vec<Relocation<InMemorySectionIndex>>,
    ) {
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

        (section_table, symbol_table, references)
    }
}
