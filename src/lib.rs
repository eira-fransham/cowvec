use std::ops::Deref;
use std::{fmt, mem, ptr, slice, str};

#[derive(Clone)]
pub struct CowStr<'a>(CowVec<'a, u8>);

impl<'a> CowStr<'a> {
    #[inline]
    pub fn borrowed(b: &'a str) -> Self {
        CowStr(CowVec::borrowed(b.as_bytes()))
    }

    #[inline]
    pub fn owned(v: String) -> Self {
        CowStr(CowVec::owned(v.into_bytes()))
    }

    #[inline]
    pub fn into_owned(self) -> String {
        unsafe { String::from_utf8_unchecked(self.0.into_owned()) }
    }

    #[inline]
    pub fn try_owned(self) -> Option<String> {
        self.0
            .try_owned()
            .map(|v| unsafe { String::from_utf8_unchecked(v) })
    }
}

impl<'a> PartialEq<&'a str> for CowStr<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<String> for CowStr<'a> {
    fn eq(&self, other: &String) -> bool {
        *self == &**other
    }
}

impl<'a> PartialEq for CowStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        *self == other.as_ref()
    }
}

impl<'a> fmt::Debug for CowStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a> fmt::Display for CowStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a> From<&'a str> for CowStr<'a> {
    fn from(other: &'a str) -> Self {
        CowStr::borrowed(other)
    }
}

impl From<String> for CowStr<'static> {
    fn from(other: String) -> Self {
        CowStr::owned(other)
    }
}

impl<'a> AsRef<str> for CowStr<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.0.as_ref()) }
    }
}

impl<'a> Deref for CowStr<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.0.deref()) }
    }
}

pub struct CowVec<'a, T: 'a> {
    ptr: *const T,
    len: usize,
    cap: usize,
    _marker: ::std::marker::PhantomData<&'a [T]>,
}

impl<'a, T> CowVec<'a, T> {
    #[inline]
    pub fn borrowed(b: &'a [T]) -> Self {
        CowVec {
            ptr: b.as_ptr(),
            len: b.len(),
            cap: 0,
            _marker: Default::default(),
        }
    }

    #[inline]
    pub fn owned(v: Vec<T>) -> Self {
        let out = CowVec {
            ptr: v.as_ptr(),
            len: v.len(),
            cap: v.capacity(),
            _marker: Default::default(),
        };

        mem::forget(v);

        out
    }

    #[inline]
    pub fn into_owned(self) -> Vec<T>
    where
        T: Clone,
    {
        self.owned_or(|x| x, |a: &_| Vec::from(a))
    }

    #[inline]
    pub fn try_owned(self) -> Option<Vec<T>> {
        self.owned_or(Some, |_: &_| None)
    }

    fn owned_or<Out, MF, OF>(self, map: MF, or: OF) -> Out
    where
        MF: FnOnce(Vec<T>) -> Out,
        OF: FnOnce(&[T]) -> Out,
    {
        let out = if self.cap == 0 {
            or(self.as_ref())
        } else {
            map(unsafe { Vec::from_raw_parts(self.ptr as *mut T, self.len, self.cap) })
        };

        mem::forget(self);

        out
    }
}

impl<'a, T> Clone for CowVec<'a, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        if self.cap == 0 {
            CowVec { ..*self }
        } else {
            Vec::from(self.as_ref()).into()
        }
    }
}

impl<'a, T> PartialEq<&'a [T]> for CowVec<'a, T>
where
    for<'any> &'any [T]: PartialEq,
{
    fn eq(&self, other: &&[T]) -> bool {
        self.as_ref() == *other
    }
}

impl<'a, T> PartialEq<Vec<T>> for CowVec<'a, T>
where
    for<'any> &'any [T]: PartialEq,
{
    fn eq(&self, other: &Vec<T>) -> bool {
        *self == &**other
    }
}

impl<'a, T> PartialEq for CowVec<'a, T>
where
    for<'any> &'any [T]: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        *self == other.as_ref()
    }
}

impl<'a, T> fmt::Debug for CowVec<'a, T>
where
    for<'any> &'any [T]: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a, T> From<&'a [T]> for CowVec<'a, T> {
    fn from(other: &'a [T]) -> Self {
        CowVec::borrowed(other)
    }
}

impl<'a, T> From<Vec<T>> for CowVec<'a, T> {
    fn from(other: Vec<T>) -> Self {
        CowVec::owned(other)
    }
}

impl<'a, T> AsRef<[T]> for CowVec<'a, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a, T> Drop for CowVec<'a, T> {
    #[inline]
    fn drop(&mut self) {
        let _ = unsafe { ptr::read(self) }.try_owned();
    }
}

impl<'a, T> Deref for CowVec<'a, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{CowStr, CowVec};

    #[test]
    fn borrowed() {
        let arr = &[1, 2, 3, 4, 5];
        let v = CowVec::borrowed(arr);

        assert_eq!(arr, &*v);
    }

    #[test]
    fn owned() {
        let arr = vec![1, 2, 3, 4, 5];
        let v = CowVec::owned(arr.clone());

        assert_eq!(arr, &*v);
    }

    #[test]
    fn borrowed_str() {
        let msg = "Hello, world!";
        let v = CowStr::borrowed(msg);

        assert_eq!(msg, &*v);
    }

    #[test]
    fn owned_str() {
        let msg = "Hello, world!".to_owned();
        let v = CowStr::owned(msg.clone());

        assert_eq!(msg, &*v);
    }
}
