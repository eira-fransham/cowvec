use std::{str, mem, ptr, slice};
use std::ops::Deref;

pub struct CowStr<'a>(CowVec<'a, u8>);

impl<'a> CowStr<'a> {
    #[inline]
    pub fn borrowed(b: &'a str) -> Self {
        CowStr(CowVec::borrowed(b.as_bytes()))
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

impl CowStr<'static> {
    #[inline]
    pub fn owned(v: String) -> Self {
        CowStr(CowVec::owned(v.into_bytes()))
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

impl<T> CowVec<'static, T> {
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
    use super::{CowVec, CowStr};

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
