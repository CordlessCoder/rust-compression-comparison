use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::{Read, Write};

enum DeflateAlgo {
    Deflate(flate2::Compression),
    Zlib(flate2::Compression),
    GZip(flate2::Compression),
}

impl DescribeScheme for DeflateAlgo {
    fn name(&self) -> String {
        "flate2 (zlib-rs)".to_string()
    }
    fn settings(&self) -> Option<String> {
        match self {
            DeflateAlgo::Deflate(c) => Some(format!("deflate / level {}", c.level())),
            DeflateAlgo::Zlib(c) => Some(format!("zlib / level {}", c.level())),
            DeflateAlgo::GZip(c) => Some(format!("gzip / level {}", c.level())),
        }
    }
}

impl Compressor for DeflateAlgo {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        match self {
            DeflateAlgo::Deflate(level) => {
                let mut encoder = flate2::write::DeflateEncoder::new(vec![], *level);
                encoder
                    .write_all(data)
                    .context("deflate compression failed")?;
                encoder.finish().context("deflate compression failed")
            }
            DeflateAlgo::Zlib(level) => {
                let mut encoder = flate2::write::ZlibEncoder::new(vec![], *level);
                encoder
                    .write_all(data)
                    .context("deflate compression failed")?;
                encoder.finish().context("zlib compression failed")
            }
            DeflateAlgo::GZip(level) => {
                let mut encoder = flate2::write::GzEncoder::new(vec![], *level);
                encoder
                    .write_all(data)
                    .context("deflate compression failed")?;
                encoder.finish().context("gzip compression failed")
            }
        }
    }
}

impl Decompressor for DeflateAlgo {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        match self {
            DeflateAlgo::Deflate(_) => {
                let mut decoder = flate2::read::DeflateDecoder::new(src);
                decoder
                    .read_exact(dst)
                    .context("deflate decompression failed")?;
                let mut tmp = [0u8];
                match decoder.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "deflate decompression failed: dst too short",
                    )),
                }
            }
            DeflateAlgo::Zlib(_) => {
                let mut decoder = flate2::read::ZlibDecoder::new(src);
                decoder
                    .read_exact(dst)
                    .context("deflate decompression failed")?;
                let mut tmp = [0u8];
                match decoder.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "deflate decompression failed: dst too short",
                    )),
                }
            }
            DeflateAlgo::GZip(_) => {
                let mut decoder = flate2::read::GzDecoder::new(src);
                decoder
                    .read_exact(dst)
                    .context("deflate decompression failed")?;
                let mut tmp = [0u8];
                match decoder.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "deflate decompression failed: dst too short",
                    )),
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    for level in 0..=9 {
        schemes.push(DeflateAlgo::Deflate(flate2::Compression::new(level)));
        schemes.push(DeflateAlgo::Zlib(flate2::Compression::new(level)));
        schemes.push(DeflateAlgo::GZip(flate2::Compression::new(level)));
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
