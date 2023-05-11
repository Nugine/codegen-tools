use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

use anyhow::Result;
use bincode::Options;
use camino::Utf8Path;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn default<T: Default>() -> T {
    Default::default()
}

pub fn write_json<T: Serialize>(path: impl AsRef<Utf8Path>, value: &T) -> Result<()> {
    let output = File::create(path.as_ref())?;
    let output = BufWriter::with_capacity(4 * 1024 * 1024, output);
    serde_json::to_writer_pretty(output, value)?;
    Ok(())
}

pub fn write_bincode<T: Serialize>(path: impl AsRef<Utf8Path>, value: &T) -> Result<()> {
    let output = File::create(path.as_ref())?;
    let output = BufWriter::with_capacity(4 * 1024 * 1024, output);
    let bincode = bincode::DefaultOptions::new();
    bincode.serialize_into(output, value)?;
    Ok(())
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Utf8Path>) -> Result<T> {
    let input = File::open(path.as_ref())?;
    let input = BufReader::with_capacity(4 * 1024 * 1024, input);
    let value = serde_json::from_reader(input)?;
    Ok(value)
}

pub fn read_bincode<T: DeserializeOwned>(path: impl AsRef<Utf8Path>) -> Result<T> {
    let input = File::open(path.as_ref())?;
    let input = BufReader::with_capacity(4 * 1024 * 1024, input);
    let bincode = bincode::DefaultOptions::new();
    let value = bincode.deserialize_from(input)?;
    Ok(value)
}

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
