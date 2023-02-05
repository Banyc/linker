use std::collections::HashMap;

use thiserror::Error;

use super::{models::*, ResolvingRelocation, ResolvingSymbolTable};

pub fn resolve_unloadable_sections<'name, S, ST>(
    section_table: &ST,
    symbol_table: &mut ResolvingSymbolTable<'name, S>,
    other_symbol_table: SymbolTable<'name, S>,
    relocation_table: &mut Vec<ResolvingRelocation<S>>,
    other_relocation_table: Vec<Relocation<S>>,
) -> Result<(), ResolveError<S>>
where
    S: SectionIndex,
    ST: LoadableSectionTable<S>,
{
    let mut resolved_symbols = HashMap::with_capacity(other_symbol_table.len());

    // Resolve symbols
    for (index, symbol) in other_symbol_table.into_iter() {
        // Update offset
        let new_index = resolve_symbol(section_table, symbol_table, &symbol)?;

        // Mark the symbol as resolved
        resolved_symbols.insert(index, new_index);
    }

    // Resolve relocations
    for reference in other_relocation_table.into_iter() {
        let Some(new_symbol) = resolved_symbols.get(&reference.symbol) else {
            return Err(ResolveError::InvalidRelocation {
                relocation: reference,
            })
        };

        // Update relocation offset and symbol index
        let new_reference = ResolvingRelocation(Relocation {
            symbol: *new_symbol,
            offset: section_table.len(reference.section) + reference.offset,
            ..reference
        });

        // Store resolved relocation
        relocation_table.push(new_reference);

        // Mark the symbol as resolved
        resolved_symbols.insert(reference.symbol, *new_symbol);
    }

    Ok(())
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ResolveError<S>
where
    S: SectionIndex,
{
    #[error("conflict `{symbol}` symbols")]
    ConflictSymbols { symbol: String },
    #[error("invalid relocation")]
    InvalidRelocation { relocation: Relocation<S> },
}

fn resolve_symbol<'name, S>(
    section_table: &impl LoadableSectionTable<S>,
    symbol_table: &mut ResolvingSymbolTable<'name, S>,
    symbol: &Symbol<'name, S>,
) -> Result<SymbolIndex, ResolveError<S>>
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
