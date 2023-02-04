use super::SymbolIndex;

pub struct Relocation {
    pub offset: usize,
    pub typ: RelocationType,
    pub symbol: SymbolIndex,
    pub addend: isize,
}

pub enum RelocationType {
    PcRelative,
    Absolute,
}
