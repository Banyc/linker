use super::{section::SectionIndex, SymbolIndex};

pub struct Relocation<S>
where
    S: SectionIndex,
{
    pub offset: usize,
    
    /// The section that the relocation is _for_.
    pub section: S,

    pub typ: RelocationType,
    pub symbol: SymbolIndex,
    pub addend: isize,
}

pub enum RelocationType {
    PcRelative,
    Absolute,
}
