pub(crate) enum CowMut<'t, T> {
    Borrowed(&'t mut T),
    Owned(T),
}

impl<'t, T> CowMut<'t, T> {
    pub(crate) fn reborrow(&mut self) -> CowMut<'_, T> {
        match self {
            Self::Borrowed(val) => CowMut::Borrowed(*val),
            Self::Owned(ref mut val) => CowMut::Borrowed(val),
        }
    }
}

impl<T> std::ops::Deref for CowMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(val) => &*val,
            Self::Owned(ref val) => val,
        }
    }
}

impl<T> std::ops::DerefMut for CowMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(val) => val,
            Self::Owned(ref mut val) => val,
        }
    }
}
