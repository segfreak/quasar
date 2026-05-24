use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

const MAGIC: &[u8; 4] = b"FEAR";

pub fn write<T: serde::Serialize, W: Write>(value: &T, writer: W) -> std::io::Result<()> {
    let mut writer = writer;

    writer.write_all(MAGIC)?;

    let mut encoder = zstd::Encoder::new(writer, 6)?;
    bincode::serialize_into(&mut encoder, value).map_err(std::io::Error::other)?;

    encoder.finish()?;
    Ok(())
}

pub fn read<T: serde::de::DeserializeOwned, R: Read>(reader: R) -> std::io::Result<T> {
    let mut reader = reader;

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    assert_eq!(&magic, MAGIC);

    let decoder = zstd::Decoder::new(reader)?;
    let value = bincode::deserialize_from(decoder).map_err(std::io::Error::other)?;

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
