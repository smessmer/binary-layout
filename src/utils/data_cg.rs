use core::ops::{Deref, DerefMut, Range};

pub trait IsTrue {}
pub struct ConstBool<const B: bool>;
impl IsTrue for ConstBool<true> {}

/// Const generic version of [super::data::Data].
/// Ensures that offset calculation happen at compile time, but requires a nightly compiler.
///
/// RANGE_END is either -1 or a positive integer. If it is -1, the range is from RANGE_START to the end of the storage.
#[derive(Clone)]
pub struct DataCG<S, const RANGE_START: usize, const RANGE_END: isize>
where
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    storage: S,
}

impl<S, const RANGE_START: usize, const RANGE_END: isize> DataCG<S, RANGE_START, RANGE_END>
where
    S: AsRef<[u8]>,
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    // TODO static_assert that RANGE_START < RANGE_END if RANGE_END != -1

    const fn range_end() -> Option<usize> {
        parse_range_end_arg(RANGE_END)
    }

    fn region(&self) -> Range<usize> {
        match Self::range_end() {
            Some(end) => RANGE_START..end,
            None => RANGE_START..self.storage.as_ref().len() as usize,
        }
    }

    /// Return the length of the [Data] instance (or if it is a subregion, length of the subregion)
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.region().len()
    }

    /// Returns true if the [Data] instance contains data and false if it has a zero length.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.region().is_empty()
    }

    /// Return a [Data] instance that semantically only represents a subregion of the original instance.
    /// Using any data accessors like `AsRef<[u8]>` or `AsMut<[u8]>` on the new instance will behave
    /// as if the instance only owned the subregion.
    ///
    /// Creating subregions is super fast and does not incur a copy.
    /// Note, however, that this is implemented by keeping all of the original data in memory and just
    /// changing the behavior of the accessors. The memory will only be freed once the subregion instance
    /// gets dropped.
    #[inline]
    pub fn into_subregion<const SUBRANGE_START: usize, const SUBRANGE_END: isize>(
        self,
    ) -> DataCG<
        S,
        { offset_range_start(RANGE_START, SUBRANGE_START) },
        { offset_range_end(RANGE_START, RANGE_END, SUBRANGE_END) },
    >
    where
        ConstBool<
            {
                range_is_valid(
                    offset_range_start(RANGE_START, SUBRANGE_START),
                    offset_range_end(RANGE_START, RANGE_END, SUBRANGE_END),
                )
            },
        >: IsTrue,
        ConstBool<
            { subrange_is_within_main_range(RANGE_START, RANGE_END, SUBRANGE_START, SUBRANGE_END) },
        >: IsTrue,
    {
        DataCG {
            storage: self.storage,
        }
    }
}

impl<S> From<S> for DataCG<S, 0, -1>
where
    S: AsRef<[u8]>,
{
    /// Create a new [Data] object from a given `Vec<[u8]>` allocation.
    #[inline(always)]
    fn from(data: S) -> Self {
        Self { storage: data }
    }
}

impl<S, const RANGE_START: usize, const RANGE_END: isize> AsRef<[u8]>
    for DataCG<S, RANGE_START, RANGE_END>
where
    S: AsRef<[u8]>,
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.storage.as_ref()[self.region()]
    }
}

impl<S, const RANGE_START: usize, const RANGE_END: isize> AsMut<[u8]>
    for DataCG<S, RANGE_START, RANGE_END>
where
    S: AsRef<[u8]> + AsMut<[u8]>,
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        let region = self.region();
        &mut self.storage.as_mut()[region]
    }
}

// TODO Test
impl<S, const RANGE_START: usize, const RANGE_END: isize> Deref
    for DataCG<S, RANGE_START, RANGE_END>
where
    S: AsRef<[u8]>,
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

// TODO Test
impl<S, const RANGE_START: usize, const RANGE_END: isize> DerefMut
    for DataCG<S, RANGE_START, RANGE_END>
where
    S: AsRef<[u8]> + AsMut<[u8]>,
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<'a, const RANGE_START: usize, const RANGE_END: isize> DataCG<&'a [u8], RANGE_START, RANGE_END>
where
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    /// Transform the [Data] object into a slice for the data pointed to.
    /// This also extracts the lifetime and can be useful to get an object
    /// whose lifetime isn't bound to the view or any local object anymore,
    /// but instead bound to the original storage of the View.
    ///
    /// Example:
    /// ---------------
    /// ```
    /// use binary_layout::binary_layout;
    ///
    /// binary_layout!(my_layout, LittleEndian, {
    ///   field: u16,
    ///   data: [u8],
    /// });
    ///
    /// fn func(input: &[u8]) -> &[u8] {
    ///   let view = my_layout::View::new(input);
    ///   // Returning `view.data_mut()` doesn't work because its lifetime is bound to the local `view` object.
    ///   // But we can return the following
    ///   view.into_data().into_slice()
    /// }
    ///
    /// let data = vec![0; 1024];
    /// assert_eq!(0, func(&data)[0]);
    /// ```
    pub fn into_slice(self) -> &'a [u8] {
        &self.storage[self.region()]
    }
}

impl<'a, const RANGE_START: usize, const RANGE_END: isize>
    DataCG<&'a mut [u8], RANGE_START, RANGE_END>
where
    ConstBool<{ range_is_valid(RANGE_START, RANGE_END) }>: IsTrue,
{
    /// Transform the [Data] object into a slice for the data pointed to.
    /// This also extracts the lifetime and can be useful to get an object
    /// whose lifetime isn't bound to the view or any local object anymore,
    /// but instead bound to the original storage of the View.
    ///
    /// Example:
    /// ---------------
    /// ```
    /// use binary_layout::binary_layout;
    ///
    /// binary_layout!(my_layout, LittleEndian, {
    ///   field: u16,
    ///   data: [u8],
    /// });
    ///
    /// fn func(input: &mut [u8]) -> &mut [u8] {
    ///   let view = my_layout::View::new(input);
    ///   // Returning `view.data_mut()` doesn't work because its lifetime is bound to the local `view` object.
    ///   // But we can return the following
    ///   view.into_data().into_slice()
    /// }
    ///
    /// let mut data = vec![0; 1024];
    /// func(&mut data)[0] = 5;
    /// ```
    pub fn into_slice(self) -> &'a mut [u8] {
        let region = self.region();
        &mut self.storage[region]
    }
}

pub const fn offset_range_start(range_start: usize, subrange_start: usize) -> usize {
    range_start + subrange_start
}

pub const fn offset_range_end(range_start: usize, range_end: isize, subrange_end: isize) -> isize {
    unparse_range_end_arg(match parse_range_end_arg(subrange_end) {
        Some(subrange_end) => Some(range_start + subrange_end),
        None => parse_range_end_arg(range_end),
    })
}

// TODO Once Rust supports it, we should just make the const generic parameter an Option instead of using -1.
const fn parse_range_end_arg(range_end: isize) -> Option<usize> {
    if range_end == -1 {
        None
    } else {
        Some(range_end as usize)
    }
}

const fn unparse_range_end_arg(range_end: Option<usize>) -> isize {
    match range_end {
        Some(end) => end as isize,
        None => -1,
    }
}

pub const fn range_is_valid(range_start: usize, range_end: isize) -> bool {
    match parse_range_end_arg(range_end) {
        Some(range_end) => range_start <= range_end,
        None => true,
    }
}

// TODO Test
pub const fn subrange_is_within_main_range(
    main_range_start: usize,
    main_range_end: isize,
    sub_range_start: usize,
    sub_range_end: isize,
) -> bool {
    let main_range_end = match parse_range_end_arg(main_range_end) {
        Some(end) => end,
        None => return true,
    };

    if main_range_start + sub_range_start > main_range_end {
        return false;
    }
    match parse_range_end_arg(sub_range_end) {
        Some(sub_range_end) => main_range_start + sub_range_end <= main_range_end,
        None => true,
    }
}
