use std::marker::PhantomData;

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StackVec4<A> {
    pub len: usize,
    pub data: [Option<A>; 4],
}

impl<A> Default for StackVec4<A> {
    fn default() -> Self {
        StackVec4 {
            len: 0,
            data: [None, None, None, None],
        }
    }
}

impl<A> StackVec4<A> {
    #[inline(always)]
    pub fn push(&mut self, val: A) {
        unsafe {
            *self.data.get_unchecked_mut(self.len) = Some(val);
        }
        self.len += 1;
    }
}

impl<'de, A> Deserialize<'de> for StackVec4<A>
where
    A: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StackVecVisitor4(PhantomData))
    }
}

struct StackVecVisitor4<A>(PhantomData<A>);

impl<'de, A> Visitor<'de> for StackVecVisitor4<A>
where
    A: Deserialize<'de>,
{
    type Value = StackVec4<A>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        let mut new_arrayvec: StackVec4<A> = Default::default();

        let mut idx = 0usize;
        while let Some(value) = seq.next_element()? {
            new_arrayvec.push(value);
            idx = idx + 1;
        }

        Ok(new_arrayvec)
    }
}
