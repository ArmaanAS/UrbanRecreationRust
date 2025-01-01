use std::{marker::PhantomData, mem::swap};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use tinyvec::Array;

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
    pub fn push_(&mut self, val: A) {
        unsafe {
            *self.data.get_unchecked_mut(self.len) = Some(val);
        }
        self.len += 1;
    }
    #[inline(always)]
    pub fn push(&mut self, val: A) {
        assert!(self.len < 4);

        self.data[self.len] = Some(val);
        self.len += 1;
    }
    #[inline(always)]
    pub fn push_1_safe(&mut self, val: A) {
        self.data[self.len & 3] = Some(val);
        self.len += 1;
    }
    #[inline(always)]
    pub fn remove(&mut self, index: usize) {
        self.len -= 1;
        if index == self.len {
            self.data[index] = None;
        } else {
            // Shift any values after the removed index down
            for i in index..self.len {
                // self.data[i] = self.data[i + 1];
                let right = self.data[i + 1].take();
                self.data[i] = right;
            }
        }
    }
    #[inline(always)]
    pub fn compact(&mut self) {
        let mut new_data = [None, None, None, None];
        let mut new_len = 0;
        for i in 0..self.len {
            if let Some(val) = self.data[i].take() {
                new_data[new_len] = Some(val);
                new_len += 1;
            }
        }

        swap(&mut self.data, &mut new_data);
        self.len = new_len;
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

#[test]
fn test_stackvec4() {
    let mut vec = StackVec4::<u8>::default();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);

    assert_eq!(vec.len, 4);
    assert_eq!(vec.data, [Some(1), Some(2), Some(3), Some(4)]);

    vec.remove(1);
    assert_eq!(vec.len, 3);
    assert_eq!(vec.data, [Some(1), Some(3), Some(4), None]);

    vec.remove(0);
    assert_eq!(vec.len, 2);
    assert_eq!(vec.data, [Some(3), Some(4), None, None]);

    vec.remove(1);
    assert_eq!(vec.len, 1);
    assert_eq!(vec.data, [Some(3), None, None, None]);

    vec.push(2);
    vec.push(1);
    vec.push(0);

    vec.data[2] = None;
    vec.compact();

    assert_eq!(vec.len, 3);
    assert_eq!(vec.data, [Some(3), Some(2), Some(0), None]);
}
