use core::convert::Into;

/// Owned Path Object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedPath
{
    path: String
}

impl OwnedPath
{
    /// Allocate a path on the heap
    pub fn new<T: Into<String>>(data: T) -> Self
    {
        Self
        {
            path: data.into()
        }
    }
}

/// Path Buffer Object
type PathBuffer<'a> = &'a OwnedPath;

impl<'a> core::iter::Iterator for PathBuffer<'a>
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item>
    {
        todo!()
    }
}
