use crate::{ByteOrder, Cuisiner, CuisinerError};

impl<const N: usize, T: Cuisiner> Cuisiner for [T; N] {
    type Raw<B: ByteOrder> = [T::Raw<B>; N];

    fn try_from_raw<B: ByteOrder>(raw: Self::Raw<B>) -> Result<Self, CuisinerError> {
        // SAFETY: Const parameters ensure that every zeroed item will be overwritten before being
        // read.
        let mut out = unsafe { std::mem::zeroed::<Self>() };

        for (raw, out) in raw.into_iter().zip(&mut out) {
            *out = T::try_from_raw(raw)?;
        }

        Ok(out)
    }

    fn try_to_raw<B: ByteOrder>(self) -> Result<Self::Raw<B>, CuisinerError> {
        // SAFETY: Const parameters ensure that every zeroed item will be overwritten before being
        // read.
        let mut out = unsafe { std::mem::zeroed::<Self::Raw<B>>() };

        for (raw, out) in self.into_iter().zip(&mut out) {
            *out = T::try_to_raw(raw)?;
        }

        Ok(out)
    }
}

#[cfg(test)]
mod test {
    use crate::BigEndian;

    use super::*;

    #[test]
    fn array() {
        let arr = [0xabcdu32, 0xef01, 0x2345];
        let bytes = arr.to_bytes::<BigEndian>().unwrap();
        let parsed = <[u32; 3]>::from_bytes::<BigEndian>(&bytes).unwrap();
        assert_eq!(arr, parsed);
    }
}
