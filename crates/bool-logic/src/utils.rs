use std::iter;
use std::mem;

pub fn replace_with<T>(place: &mut T, dummy: T, f: impl FnOnce(T) -> T) {
    let dummy = mem::replace(place, dummy);
    let ans = f(dummy);
    *place = ans;
}

// TODO: move to rust_utils
pub fn filter_map_collect<C, T, I, F>(iterable: I, f: F) -> C
where
    I: IntoIterator,
    F: FnMut(I::Item) -> Option<T>,
    C: FromIterator<T>,
{
    iterable.into_iter().filter_map(f).collect()
}

/// TODO: move to rust_utils
pub fn remove_if<T>(v: &mut Vec<T>, mut f: impl FnMut(&mut T) -> bool) {
    v.retain_mut(|x| !f(x))
}

/// TODO: move to rust_utils
pub fn drain_filter<'a, T, F>(v: &'a mut Vec<T>, mut f: F) -> impl Iterator<Item = T> + 'a
where
    F: FnMut(&mut T) -> bool + 'a,
{
    let mut i = 0;
    iter::from_fn(move || {
        while i < v.len() {
            if f(&mut v[i]) {
                return Some(v.remove(i));
            } else {
                i += 1;
            }
        }
        None
    })
}
