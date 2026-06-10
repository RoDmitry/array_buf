use ::core::{ops::Deref, ptr::copy_nonoverlapping};

/// Mimics little-endian unsigned integer with dynamic size.
/// Somewhat similar to `arrayvec::ArrayVec`, but the inner array is initialized.
#[derive(Debug, Clone, Copy)]
pub struct BytesArray<const CAP: usize> {
    len: usize,
    pub arr: [u8; CAP],
}

impl<const CAP: usize> Default for BytesArray<CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> BytesArray<CAP> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            len: 0,
            arr: [0; CAP],
        }
    }

    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Free, available space.
    #[inline(always)]
    pub const fn remaining(&self) -> usize {
        CAP - self.len
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { self.arr.get_unchecked(..self.len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { self.arr.get_unchecked_mut(..self.len) }
    }

    /// # Safety
    ///
    /// Must not be full.
    #[inline]
    pub unsafe fn push_unchecked(&mut self, v: u8) {
        debug_assert!(self.len < CAP);
        *unsafe { self.arr.get_unchecked_mut(self.len) } = v;
        self.len += 1;
    }

    #[inline]
    pub fn push(&mut self, v: u8) {
        assert!(self.len < CAP);
        unsafe { self.push_unchecked(v) };
    }

    /// # Safety
    ///
    /// Must not exceed capacity.
    #[inline]
    pub const unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= CAP);
        self.len = len;
    }

    /// Not recommended. Can be a lot slower than native int.
    /// Little-endian style.
    /// Returns true if overflow happened.
    #[inline]
    pub fn increment_wrapping(&mut self) -> bool {
        for byte in self.as_mut_slice().iter_mut() {
            let (val, overflow) = byte.overflowing_add(1);
            *byte = val;
            if !overflow {
                return false; // No carry needed, we're done
            }
            // overflow == true → byte wrapped to 0x00, carry continues
        }
        // Every byte overflowed
        if self.len < CAP {
            unsafe { self.push_unchecked(1) };
            false
        } else {
            self.len = 0;
            true
        }
    }

    /// Not recommended. Can be a lot slower than native int.
    /// Little-endian style.
    #[inline]
    pub fn increment(&mut self) {
        if self.increment_wrapping() {
            panic!("BytesArray: overflowed on increment");
        }
    }

    /// Silently skipping bytes which do not fit.
    #[inline]
    pub fn extend_from_slice(&mut self, src: &[u8]) {
        let rem = self.remaining();
        if rem == 0 {
            return;
        }
        let copy_len = if rem < src.len() { rem } else { src.len() };

        // Copy the bytes.
        // SAFETY: The slices cannot overlap because mutable references are exclusive.
        unsafe {
            let dst_ptr = self.arr.get_unchecked_mut(self.len);
            copy_nonoverlapping(src.as_ptr(), dst_ptr, copy_len);
        }
        self.len += copy_len;
    }
}

impl<const CAP: usize> AsRef<[u8]> for BytesArray<CAP> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const CAP: usize> Deref for BytesArray<CAP> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

const USIZE: usize = ::core::mem::size_of::<usize>();
impl<const CAP: usize> From<usize> for BytesArray<CAP> {
    #[inline]
    fn from(v: usize) -> Self {
        const {
            assert!(
                CAP >= USIZE,
                "BytesArray: too small capacity to convert from usize"
            );
        }

        let len = USIZE - (v.leading_zeros() / 8) as usize;

        let bytes: [u8; USIZE] = v.to_le_bytes();
        let mut arr = [0; CAP];
        arr[..USIZE].copy_from_slice(&bytes);

        Self { len, arr }
    }
}

impl<const CAP: usize> From<u64> for BytesArray<CAP> {
    #[inline]
    fn from(v: u64) -> Self {
        const {
            assert!(
                CAP >= 8,
                "BytesArray: capacity must be at least 8 to convert from u64"
            );
        }

        let len = (8 - v.leading_zeros() / 8) as usize;

        let bytes = v.to_le_bytes();
        let mut arr = [0; CAP];
        arr[..8].copy_from_slice(&bytes);

        Self { len, arr }
    }
}
