use binrw::{BinRead, BinResult, BinWrite};

pub struct FixedString<const N: usize>(pub(crate) [u8; N]);

impl<const N: usize> std::fmt::Display for FixedString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FixedString(bytes) = self;
        f.write_str(String::from_utf8_lossy(bytes).as_ref())
    }
}

impl<const N: usize> From<FixedString<N>> for String {
    fn from(FixedString(bytes): FixedString<N>) -> Self {
        String::from_utf8_lossy(&bytes).to_string()
    }
}

impl<const N: usize> From<String> for FixedString<N> {
    fn from(value: String) -> Self {
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(&value.as_bytes()[..N]);
        FixedString(bytes)
    }
}

impl<const N: usize> BinRead for FixedString<N> {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(FixedString(<[u8; N]>::read_options(reader, endian, ())?))
    }
}

impl<const N: usize> BinWrite for FixedString<N> {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        let FixedString(bytes) = self;
        bytes.write_options(writer, endian, ())
    }
}
