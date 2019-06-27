//! Effect

/// An effect in the EEE model.
#[derive(Clone)]
pub enum Effect {
    ///
    Bytes(Vec<u8>),
    ///
    Bytes2([u8; 2]),
    ///
    Bytes6([u8; 6]),
    ///
    Bytes18([u8; 18]),
    ///
    Bytes54([u8; 54]),
    ///
    Bytes162([u8; 162]),
    ///
    Bytes486([u8; 486]),
    ///
    Trytes(Vec<char>),
    ///
    Trytes3([char; 3]),
    ///
    Trytes9([char; 9]),
    ///
    Trytes27([char; 27]),
    ///
    Trytes81([char; 81]),
    ///
    Trytes243([char; 243]),
    ///
    Trytes729([char; 729]),
    ///
    Trits(Vec<i8>),
    ///
    Trits9([i8; 9]),
    ///
    Trits27([i8; 27]),
    ///
    Trits81([i8; 81]),
    ///
    Trits243([i8; 243]),
    ///
    Trits729([i8; 729]),
    ///
    Trits2187([i8; 2187]),
    /// ASCII text
    Ascii(String),
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Effect::Bytes2(bytes) => write!(f, "[{}, {}]", bytes[0], bytes[1]),
            Effect::Ascii(text) => write!(f, "{}", text),
            _ => unimplemented!(),
        }
    }
}
