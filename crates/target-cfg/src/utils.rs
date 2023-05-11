pub fn map_collect<I, T, U, C>(iter: I, f: impl FnMut(T) -> U) -> C
where
    I: IntoIterator<Item = T>,
    C: FromIterator<U>,
{
    iter.into_iter().map(f).collect()
}

pub fn map_collect_vec<I, T, U>(iter: I, f: impl FnMut(T) -> U) -> Vec<U>
where
    I: IntoIterator<Item = T>,
{
    map_collect(iter, f)
}
