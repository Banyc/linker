use super::{section::SectionIndex, SymbolIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relocation<S>
where
    S: SectionIndex,
{
    pub offset: usize,

    /// The section that the symbol reference is in.
    pub section: S,

    pub typ: RelocationType,
    pub symbol: SymbolIndex,
    pub addend: isize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RelocationType {
    PcRelative,
    Absolute,
}
