use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

pub fn write<T, W>(value: &T, writer: W) -> io::Result<()>
where
    T: serde::Serialize,
    W: Write,
{
    let mut encoder = zstd::Encoder::new(writer, 6)?;
    bincode::serialize_into(&mut encoder, value).map_err(io::Error::other)?;
    encoder.finish()?;
    Ok(())
}

pub fn read<T, R>(reader: R) -> io::Result<T>
where
    T: serde::de::DeserializeOwned,
    R: Read,
{
    let mut decoder = zstd::Decoder::new(reader)?;
    let value = bincode::deserialize_from(&mut decoder).map_err(io::Error::other)?;
    Ok(value)
}

pub fn write_to_file<T: serde::Serialize>(value: &T, path: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    write(value, writer)
}

pub fn load_from_file<T: serde::de::DeserializeOwned>(path: &str) -> io::Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    read(reader)
}
