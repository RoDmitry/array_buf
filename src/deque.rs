use ::core::{fmt::Debug, mem::MaybeUninit, ptr};

pub(crate) struct ArrayDequeBase<T, const CAP: usize> {
    arr: [MaybeUninit<T>; CAP],
    start: usize,
    end: usize,
    full: bool,
}

impl<T: Debug, const CAP: usize> Debug for ArrayDequeBase<T, CAP> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ArrayDequeBase")
            .field("arr", &self.as_slices())
            .field("start", &self.start)
            .field("end", &self.end)
            .field("full", &self.full)
            .finish()
    }
}

impl<T: Clone, const CAP: usize> Clone for ArrayDequeBase<T, CAP> {
    fn clone(&self) -> Self {
        let idx_iter = if self.is_contiguous_any_order() {
            let range = if self.full {
                0..CAP
            } else {
                self.start..self.end
            };
            // added empty chain to make it compile (produces an iter of the same type)
            (range).chain(0..0)
        } else {
            (0..self.end).chain(self.start..CAP)
        };

        let mut new_arr: [MaybeUninit<T>; CAP] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in idx_iter {
            let val = unsafe { self.arr.get_unchecked(i).assume_init_ref() };
            let new_val = unsafe { new_arr.get_unchecked_mut(i) };
            *new_val = MaybeUninit::new(val.clone());
        }

        Self {
            arr: new_arr,
            start: self.start,
            end: self.end,
            full: self.full,
        }
    }
}
impl<T: Copy, const CAP: usize> Copy for ArrayDequeBase<T, CAP> {}

impl<T, const CAP: usize> Default for ArrayDequeBase<T, CAP> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> ArrayDequeBase<T, CAP> {
    /// Bits
    const MAX_IDX: usize = CAP - 1;

    /// Creates an empty `ArrayDeque`.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let buf: ArrayDeque<usize, 2> = ArrayDeque::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        const { assert!(CAP > 1) };
        const { assert!(CAP.is_power_of_two()) };
        Self {
            arr: unsafe { MaybeUninit::uninit().assume_init() },
            start: 0,
            end: 0,
            full: false,
        }
    }

    /// Returns the capacity of the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let buf: ArrayDeque<usize, 2> = ArrayDeque::new();
    ///
    /// assert_eq!(buf.capacity(), 2);
    /// ```
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Returns the number of elements in the array.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.len(), 0);
    ///
    /// buf.push_last(1).unwrap();
    ///
    /// assert_eq!(buf.len(), 1);
    /// ```
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.len(), 0);
    ///
    /// buf.push_first(-1).unwrap();
    ///
    /// assert_eq!(buf.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        if self.full {
            self.capacity()
        } else {
            self.end.wrapping_sub(self.start) & Self::MAX_IDX
        }
    }

    /// Returns true if the array contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert!(buf.is_empty());
    ///
    /// buf.push_last(1).unwrap();
    ///
    /// assert!(!buf.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end && !self.full
    }

    /// Returns true if the array is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert!(!buf.is_full());
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert!(buf.is_full());
    /// ```
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// Order can be compromised once full.
    #[inline]
    pub fn is_contiguous_any_order(&self) -> bool {
        self.start <= self.end
    }

    #[inline]
    pub fn is_contiguous(&self) -> bool {
        self.start <= self.end && !self.full
    }

    /// # Safety
    ///
    /// Must not be empty.
    #[inline]
    pub unsafe fn pop_first_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty());

        let val = unsafe { self.arr.get_unchecked(self.start).assume_init_read() };
        self.start = self.start.wrapping_add(1) & Self::MAX_IDX;
        self.full = false;
        val
    }

    /// # Safety
    ///
    /// Must not be empty.
    #[inline]
    pub unsafe fn pop_last_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty());

        self.end = self.end.wrapping_sub(1) & Self::MAX_IDX;
        self.full = false;
        unsafe { self.arr.get_unchecked(self.end).assume_init_read() }
    }

    /// Removes the first element and returns it, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.pop_first(), None);
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.pop_first(), Some(1));
    /// assert_eq!(buf.pop_first(), Some(2));
    /// assert_eq!(buf.pop_first(), None);
    /// ```
    #[inline]
    pub fn pop_first(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        Some(unsafe { self.pop_first_unchecked() })
    }

    /// Removes the last element and returns it, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.pop_last(), None);
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.pop_last(), Some(2));
    /// assert_eq!(buf.pop_last(), Some(1));
    /// assert_eq!(buf.pop_last(), None);
    /// ```
    #[inline]
    pub fn pop_last(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        Some(unsafe { self.pop_last_unchecked() })
    }

    /// # Safety
    ///
    /// Must not be full.
    #[inline]
    pub unsafe fn push_first_unchecked(&mut self, element: T) {
        debug_assert!(!self.is_full());

        self.start = self.start.wrapping_sub(1) & Self::MAX_IDX;
        let val = unsafe { self.arr.get_unchecked_mut(self.start) };
        *val = MaybeUninit::new(element);
        self.full = self.start == self.end;
    }

    /// # Safety
    ///
    /// Must not be full.
    #[inline]
    pub unsafe fn push_last_unchecked(&mut self, element: T) {
        debug_assert!(!self.is_full());

        let val = unsafe { self.arr.get_unchecked_mut(self.end) };
        *val = MaybeUninit::new(element);
        self.end = self.end.wrapping_add(1) & Self::MAX_IDX;
        self.full = self.start == self.end;
    }

    /// Add an element to the start of the deque.
    ///
    /// Return `Ok` if the push succeeds, or `Err` if the array is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    ///
    /// buf.push_first(-1);
    /// buf.push_first(-2);
    ///
    /// let overflow = buf.push_first(-3);
    ///
    /// assert!(overflow.is_err());
    /// assert_eq!(buf.first(), Some(&-2));
    /// ```
    #[inline]
    pub fn push_first(&mut self, element: T) -> Result<(), &'static str> {
        if self.is_full() {
            return Err("array is full");
        }
        unsafe { self.push_first_unchecked(element) };
        Ok(())
    }

    /// Add an element to the end of the deque.
    ///
    /// Return `Ok` if the push succeeds, or `Err` if the array is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    ///
    /// buf.push_last(1);
    /// buf.push_last(2);
    ///
    /// let overflow = buf.push_last(3);
    ///
    /// assert!(overflow.is_err());
    /// assert_eq!(buf.last(), Some(&2));
    /// ```
    #[inline]
    pub fn push_last(&mut self, element: T) -> Result<(), &'static str> {
        if self.is_full() {
            return Err("array is full");
        }
        unsafe { self.push_last_unchecked(element) };
        Ok(())
    }

    /// Provides a reference to the first element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.first(), None);
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.first(), Some(&1));
    /// ```
    #[inline]
    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.arr.get_unchecked(self.start).assume_init_ref() })
        }
    }

    /// Provides a mut reference to the first element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.first_mut(), None);
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.first_mut(), Some(&mut 1));
    /// ```
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.arr.get_unchecked_mut(self.start).assume_init_mut() })
        }
    }

    /// Provides a reference to the last element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.last(), None);
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.last(), Some(&2));
    /// ```
    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let idx = self.end.wrapping_sub(1) & Self::MAX_IDX;
            Some(unsafe { self.arr.get_unchecked(idx).assume_init_ref() })
        }
    }

    /// Provides a mut reference to the last element, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
    /// assert_eq!(buf.last_mut(), None);
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.last_mut(), Some(&mut 2));
    /// ```
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            let idx = self.end.wrapping_sub(1) & Self::MAX_IDX;
            Some(unsafe { self.arr.get_unchecked_mut(idx).assume_init_mut() })
        }
    }

    /// Returns a slice which contains the content of the inner buffer.
    ///
    /// # Safety
    ///
    /// Must be contiguous. If it's not, use `linearize()`.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[T] {
        debug_assert!(self.is_contiguous_any_order());

        if self.full {
            unsafe { self.arr.assume_init_ref() }
        } else {
            unsafe {
                self.arr
                    .get_unchecked(self.start..self.end)
                    .assume_init_ref()
            }
        }
    }

    /// Returns a slice which contains the content of the inner buffer.
    ///
    /// # Safety
    ///
    /// Must be contiguous. If it's not, use `linearize()`.
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        debug_assert!(self.is_contiguous_any_order());

        if self.full {
            unsafe { self.arr.assume_init_mut() }
        } else {
            unsafe {
                self.arr
                    .get_unchecked_mut(self.start..self.end)
                    .assume_init_mut()
            }
        }
    }

    /// Returns a pair of slices which contain, in order, the contents of the
    /// inner buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 8> = ArrayDeque::new();
    ///
    /// assert_eq!(buf.as_slices(), (&[][..], &[][..]));
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.as_slices(), (&[1, 2][..], &[][..]));
    ///
    /// buf.push_first(-1).unwrap();
    ///
    /// assert_eq!(buf.as_slices(), (&[-1][..], &[1, 2][..]));
    /// ```
    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.is_contiguous() {
            (unsafe { self.as_slice() }, &[])
        } else {
            unsafe {
                (
                    self.arr.get_unchecked(self.start..CAP).assume_init_ref(),
                    self.arr.get_unchecked(0..self.end).assume_init_ref(),
                )
            }
        }
    }

    /// Returns a pair of slices which contain, in order, the contents of the
    /// inner buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 8> = ArrayDeque::new();
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.as_mut_slices(), (&mut [1, 2][..], &mut[][..]));
    ///
    /// buf.push_first(-1);
    ///
    /// assert_eq!(buf.as_mut_slices(), (&mut[-1][..], &mut[1, 2][..]));
    /// ```
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 4> = ArrayDeque::new();
    ///
    /// assert_eq!(buf.as_mut_slices(), (&mut [][..], &mut[][..]));
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_last(2).unwrap();
    ///
    /// assert_eq!(buf.as_mut_slices(), (&mut [1, 2][..], &mut[][..]));
    ///
    /// buf.push_first(-1).unwrap();
    /// buf.push_first(-2).unwrap();
    ///
    /// assert_eq!(buf.as_mut_slices(), (&mut[-2, -1][..], &mut[1, 2][..]));
    /// ```
    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        if self.is_contiguous() {
            (unsafe { self.as_mut_slice() }, &mut [])
        } else {
            unsafe {
                let (mid, start) = self.arr.split_at_mut_unchecked(self.start);
                (
                    start.assume_init_mut(),
                    mid.get_unchecked_mut(0..self.end).assume_init_mut(),
                )
            }
        }
    }

    /// Make the buffer contiguous.
    ///
    /// The linearization may be required when interacting with external
    /// interfaces requiring contiguous slices.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<isize, 4> = ArrayDeque::new();
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_first(-1).unwrap();
    ///
    /// assert!(!buf.is_contiguous());
    ///
    /// buf.linearize();
    ///
    /// assert!(buf.is_contiguous());
    /// ```
    #[inline]
    pub fn linearize(&mut self) {
        if self.start > 0 {
            self.arr.rotate_left(self.start);
            self.end = self.end.wrapping_sub(self.start) & Self::MAX_IDX;
            self.start = 0;
        }
    }

    /// Make the buffer contiguous.
    ///
    /// The linearization may be required when interacting with external
    /// interfaces requiring contiguous slices.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<isize, 2> = ArrayDeque::new();
    ///
    /// buf.push_first(-1).unwrap();
    ///
    /// assert!(!buf.is_contiguous());
    ///
    /// buf.linearize_one();
    ///
    /// assert!(buf.is_contiguous());
    /// ```
    #[inline]
    pub fn linearize_one(&mut self) {
        if self.len() == 1 && self.start > 0 {
            unsafe {
                let data = self.arr.get_unchecked(self.start).as_ptr();
                let first = self.arr.first_mut().unwrap_unchecked();
                ptr::copy_nonoverlapping(data, first.as_mut_ptr(), 1);
            }
            self.start = 0;
            self.end = 1;
        }
    }

    /// Clears the buffer by resetting the indexes.
    #[inline]
    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
        self.full = false;
    }
}

macro_rules! reimpl_common_methods {
    ($struct_name:ident $(< $($struct_gen:tt),* $(,)? >)?) => {
        impl<T $(: $($struct_gen +)*)?, const CAP: usize> $struct_name<T, CAP> {
            #[doc = concat!("
                Creates an empty ", stringify!($struct_name), ".
                
                # Examples
                
                ```
                use array_buf::", stringify!($struct_name), ";
                
                let buf: ", stringify!($struct_name), "<usize, 2> = ", stringify!($struct_name), "::new();
                ```
            ")]
            #[inline(always)]
            pub const fn new() -> Self {
                Self(ArrayDequeBase::new())
            }

            #[doc = concat!("
                Returns the capacity of the array.
                
                # Examples
                
                ```
                use array_buf::", stringify!($struct_name), ";
                
                let buf: ", stringify!($struct_name), "<usize, 2> = ", stringify!($struct_name), "::new();
                
                assert_eq!(buf.capacity(), 2);
                ```
            ")]
            #[inline(always)]
            pub const fn capacity(&self) -> usize {
                self.0.capacity()
            }

            #[doc = concat!("
                Returns the number of elements in the array.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert_eq!(buf.len(), 0);

                buf.push_last(1).unwrap();

                assert_eq!(buf.len(), 1);
                ```

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert_eq!(buf.len(), 0);

                buf.push_first(-1).unwrap();

                assert_eq!(buf.len(), 1);
                ```
            ")]
            #[inline(always)]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc = concat!("
                Returns true if the array contains no elements.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert!(buf.is_empty());

                buf.push_last(1).unwrap();

                assert!(!buf.is_empty());
                ```
            ")]
            #[inline(always)]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("
                Returns true if the array is full.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert!(!buf.is_full());

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert!(buf.is_full());
                ```
            ")]
            #[inline(always)]
            pub fn is_full(&self) -> bool {
                self.0.is_full()
            }

            /// Order can be compromised once full.
            #[inline(always)]
            pub fn is_contiguous_any_order(&self) -> bool {
                self.0.is_contiguous_any_order()
            }

            #[inline(always)]
            pub fn is_contiguous(&self) -> bool {
                self.0.is_contiguous()
            }

            /// # Safety
            ///
            /// Must not be empty.
            #[inline(always)]
            pub unsafe fn pop_first_unchecked(&mut self) -> T {
                unsafe { self.0.pop_first_unchecked() }
            }

            /// # Safety
            ///
            /// Must not be empty.
            #[inline(always)]
            pub unsafe fn pop_last_unchecked(&mut self) -> T {
                unsafe { self.0.pop_last_unchecked() }
            }

            #[doc = concat!("
                Removes the first element and returns it, or `None` if empty.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert_eq!(buf.pop_first(), None);

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.pop_first(), Some(1));
                assert_eq!(buf.pop_first(), Some(2));
                assert_eq!(buf.pop_first(), None);
                ```
            ")]
            #[inline(always)]
            pub fn pop_first(&mut self) -> Option<T> {
                self.0.pop_first()
            }

            #[doc = concat!("
                Removes the last element and returns it, or `None` if empty.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert_eq!(buf.pop_last(), None);

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.pop_last(), Some(2));
                assert_eq!(buf.pop_last(), Some(1));
                assert_eq!(buf.pop_last(), None);
                ```
            ")]
            #[inline(always)]
            pub fn pop_last(&mut self) -> Option<T> {
                self.0.pop_last()
            }

            /// # Safety
            ///
            /// Must not be full.
            #[inline(always)]
            pub unsafe fn push_first_unchecked(&mut self, element: T) {
                unsafe { self.0.push_first_unchecked(element) }
            }

            /// # Safety
            ///
            /// Must not be full.
            #[inline(always)]
            pub unsafe fn push_last_unchecked(&mut self, element: T) {
                unsafe { self.0.push_last_unchecked(element) }
            }

            #[doc = concat!("
                Add an element to the start of the deque.

                Return `Ok` if the push succeeds, or `Err` if the array is full.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();

                buf.push_first(-1);
                buf.push_first(-2);

                let overflow = buf.push_first(-3);

                assert!(overflow.is_err());
                assert_eq!(buf.first(), Some(&-2));
                ```
            ")]
            #[inline(always)]
            pub fn push_first(&mut self, element: T) -> Result<(), &'static str> {
                self.0.push_first(element)
            }

            #[doc = concat!("
                Add an element to the end of the deque.

                Return `Ok` if the push succeeds, or `Err` if the array is full.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();

                buf.push_last(1);
                buf.push_last(2);

                let overflow = buf.push_last(3);

                assert!(overflow.is_err());
                assert_eq!(buf.last(), Some(&2));
                ```
            ")]
            #[inline(always)]
            pub fn push_last(&mut self, element: T) -> Result<(), &'static str> {
                self.0.push_last(element)
            }

            #[doc = concat!("
                Provides a reference to the first element, or `None` if empty.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert_eq!(buf.first(), None);

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.first(), Some(&1));
                ```
            ")]
            #[inline(always)]
            pub fn first(&self) -> Option<&T> {
                self.0.first()
            }

            #[doc = concat!("
                Provides a mut reference to the first element, or `None` if empty.

                # Examples

                ```
                use array_buf::ArrayDeque;

                let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
                assert_eq!(buf.first_mut(), None);

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.first_mut(), Some(&mut 1));
                ```
            ")]
            #[inline]
            pub fn first_mut(&mut self) -> Option<&mut T> {
                self.0.first_mut()
            }

            #[doc = concat!("
                Provides a reference to the last element, or `None` if empty.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 2> = ", stringify!($struct_name), "::new();
                assert_eq!(buf.last(), None);

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.last(), Some(&2));
                ```
            ")]
            #[inline(always)]
            pub fn last(&self) -> Option<&T> {
                self.0.last()
            }

            #[doc = concat!("
                Provides a mut reference to the last element, or `None` if empty.

                # Examples

                ```
                use array_buf::ArrayDeque;

                let mut buf: ArrayDeque<_, 2> = ArrayDeque::new();
                assert_eq!(buf.last_mut(), None);

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.last_mut(), Some(&mut 2));
                ```
            ")]
            #[inline]
            pub fn last_mut(&mut self) -> Option<&mut T> {
                self.0.last_mut()
            }

            /// Returns a slice which contains the content of the inner buffer.
            ///
            /// # Safety
            ///
            /// Must be contiguous. If it's not, use `linearize()`.
            #[inline(always)]
            pub unsafe fn as_slice(&self) -> &[T] {
                unsafe { self.0.as_slice() }
            }

            /// Returns a slice which contains the content of the inner buffer.
            ///
            /// # Safety
            ///
            /// Must be contiguous. If it's not, use `linearize()`.
            #[inline(always)]
            pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
                unsafe { self.0.as_mut_slice() }
            }

            #[doc = concat!("
                Returns a pair of slices which contain, in order, the contents of the
                inner buffer.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 8> = ", stringify!($struct_name), "::new();

                assert_eq!(buf.as_slices(), (&[][..], &[][..]));

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.as_slices(), (&[1, 2][..], &[][..]));

                buf.push_first(-1).unwrap();

                assert_eq!(buf.as_slices(), (&[-1][..], &[1, 2][..]));
                ```
            ")]
            #[inline(always)]
            pub fn as_slices(&self) -> (&[T], &[T]) {
                self.0.as_slices()
            }

            #[doc = concat!("
                Returns a pair of slices which contain, in order, the contents of the
                inner buffer.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 8> = ", stringify!($struct_name), "::new();

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.as_mut_slices(), (&mut [1, 2][..], &mut[][..]));

                buf.push_first(-1);

                assert_eq!(buf.as_mut_slices(), (&mut[-1][..], &mut[1, 2][..]));
                ```

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<_, 4> = ", stringify!($struct_name), "::new();

                assert_eq!(buf.as_mut_slices(), (&mut [][..], &mut[][..]));

                buf.push_last(1).unwrap();
                buf.push_last(2).unwrap();

                assert_eq!(buf.as_mut_slices(), (&mut [1, 2][..], &mut[][..]));

                buf.push_first(-1).unwrap();
                buf.push_first(-2).unwrap();

                assert_eq!(buf.as_mut_slices(), (&mut[-2, -1][..], &mut[1, 2][..]));
                ```
            ")]
            #[inline(always)]
            pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
                self.0.as_mut_slices()
            }

            #[doc = concat!("
                Make the buffer contiguous.

                The linearization may be required when interacting with external
                interfaces requiring contiguous slices.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<isize, 4> = ", stringify!($struct_name), "::new();

                buf.push_last(1).unwrap();
                buf.push_first(-1).unwrap();

                assert!(!buf.is_contiguous());

                buf.linearize();

                assert!(buf.is_contiguous());
                ```
            ")]
            #[inline(always)]
            pub fn linearize(&mut self) {
                self.0.linearize()
            }

            #[doc = concat!("
                Make the buffer contiguous.

                The linearization may be required when interacting with external
                interfaces requiring contiguous slices.

                # Examples

                ```
                use array_buf::", stringify!($struct_name), ";

                let mut buf: ", stringify!($struct_name), "<isize, 2> = ", stringify!($struct_name), "::new();

                buf.push_first(-1).unwrap();

                assert!(!buf.is_contiguous());

                buf.linearize_one();

                assert!(buf.is_contiguous());
                ```
            ")]
            #[inline(always)]
            pub fn linearize_one(&mut self) {
                self.0.linearize_one()
            }
        }
    };
}

/// A fixed capacity deque for plain data (`Copy`, no `Drop`). Capacity must be in the power of two.
///
/// Can be stored directly on the stack.
///
/// The "default" usage of this as a queue is to use `push_last` to add to
/// the queue, and `pop_first` to consume from the queue.
#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct ArrayDequePlain<T: Copy, const CAP: usize>(ArrayDequeBase<T, CAP>);

reimpl_common_methods!(ArrayDequePlain<Copy>);

impl<T: Copy, const CAP: usize> ArrayDequePlain<T, CAP> {
    /// Clears the buffer by resetting the indexes.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDequePlain;
    ///
    /// let mut buf: ArrayDequePlain<_, 4> = ArrayDequePlain::new();
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_first(-1).unwrap();
    /// buf.clear();
    ///
    /// assert!(buf.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// A fixed capacity deque. Capacity must be in the power of two.
/// If you have plain data, better use `ArrayDequePlain`.
///
/// Can be stored directly on the stack.
///
/// The "default" usage of this as a queue is to use `push_last` to add to
/// the queue, and `pop_first` to consume from the queue.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct ArrayDeque<T, const CAP: usize>(ArrayDequeBase<T, CAP>);

impl<T: Clone, const CAP: usize> Clone for ArrayDeque<T, CAP> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

reimpl_common_methods!(ArrayDeque);

impl<T, const CAP: usize> ArrayDeque<T, CAP> {
    #[inline]
    fn drop_arr_vals(&mut self) {
        let (mem_right, mem_left) = self.as_mut_slices();
        // iterating in order of incrementing mem address
        for v in mem_left.iter_mut().chain(mem_right) {
            unsafe { ptr::drop_in_place(v) };
        }
    }

    /// Clears the buffer by dropping and resetting the indexes.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_buf::ArrayDeque;
    ///
    /// let mut buf: ArrayDeque<_, 4> = ArrayDeque::new();
    ///
    /// buf.push_last(1).unwrap();
    /// buf.push_first(-1).unwrap();
    /// buf.clear();
    ///
    /// assert!(buf.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.drop_arr_vals();
        self.0.clear();
    }
}

impl<T, const CAP: usize> Drop for ArrayDeque<T, CAP> {
    #[inline(always)]
    fn drop(&mut self) {
        self.drop_arr_vals();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy() {
        let mut a = ArrayDequePlain::<i32, 8>::new();
        a.push_last(1).unwrap();
        a.push_last(2).unwrap();

        // contiguous
        let mut b = a;
        assert_eq!(a.pop_first(), Some(1));
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 2);
        assert_eq!(b.pop_first(), Some(1));

        // not contiguous
        a.push_first(-1).unwrap();
        a.push_first(-2).unwrap();
        let mut c = a;
        assert_eq!(a.pop_first(), Some(-2));
        assert_eq!(a.len(), 2);
        assert_eq!(c.len(), 3);
        assert_eq!(c.pop_first(), Some(-2));
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn test_clone() {
        let mut a = ArrayDeque::<String, 8>::new();
        a.push_last("1".to_owned()).unwrap();

        // contiguous
        let mut b = a.clone();
        a.first_mut().unwrap().push('x');
        assert_eq!(a.pop_first(), Some("1x".to_owned()));
        assert_eq!(b.first().map(|s| s.as_str()), Some("1"));

        // not contiguous
        b.push_first("-1".to_owned()).unwrap();
        let c = b.clone();
        b.first_mut().unwrap().push('x');
        assert_eq!(b.pop_first(), Some("-1x".to_owned()));
        assert_eq!(c.first().map(|s| s.as_str()), Some("-1"));
    }

    #[test]
    fn test_linearize_one_skip() {
        let mut buf: ArrayDeque<isize, 2> = ArrayDeque::new();

        buf.push_last(1).unwrap();
        buf.linearize_one();

        assert!(buf.is_contiguous());
    }
}
