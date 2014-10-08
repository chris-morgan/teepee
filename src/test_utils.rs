use std::io::{MemReader, IoResult, SeekStyle};


/// A Reader that will only yield one byte at a time.
///
/// This is only really meaningful for testing, when you have things that might
/// have accidentally slipped into the assumption that data is always available
/// in chunks, such as benevolent HTTP. But a malicious client or server could
/// conceivably manipulate a bad HTTP parser that didn't take proper care of
/// this.
pub struct TrickleReader<R: Reader> {
    reader: R,
}

impl<R: Reader> TrickleReader<R> {
    /// Creates a new `TrickleReader` based on the given reader.
    #[inline]
    pub fn new(reader: R) -> TrickleReader<R> {
        TrickleReader {
            reader: reader,
        }
    }

    /// Unwraps this `TrickleReader`, returning the underlying reader.
    #[inline]
    pub fn unwrap(self) -> R { self.reader }
}

impl TrickleReader<MemReader> {
    /// Tests whether this reader has read all bytes in its buffer.
    ///
    /// If `true`, then this will no longer return bytes from `read`.
    #[inline]
    pub fn eof(&self) -> bool { self.reader.eof() }
}

impl<R: Reader> Reader for TrickleReader<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        if buf.len() == 0 {
            Ok(0)
        } else {
            self.reader.read(buf.slice_mut(0, 1))
            // FIXME: switch from .slice_mut(0, 1) to [0..1] when it works.
        }
    }
}

impl<R: Reader + Seek> Seek for TrickleReader<R> {
    #[inline]
    fn tell(&self) -> IoResult<u64> { self.reader.tell() }

    #[inline]
    fn seek(&mut self, pos: i64, style: SeekStyle) -> IoResult<()> {
        self.reader.seek(pos, style)
    }
}

impl<R: Reader + Buffer> Buffer for TrickleReader<R> {
    #[inline]
    fn fill_buf<'a>(&'a mut self) -> IoResult<&'a [u8]> {
        match self.reader.fill_buf() {
            Ok(o) if o.len() > 0 => Ok(o.slice(0, 1)),
            anything_else => anything_else,
        }
    }

    #[inline]
    fn consume(&mut self, amt: uint) { self.reader.consume(amt) }
}

impl<R: Reader + Writer> Writer for TrickleReader<R> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.reader.write(buf)
    }
}
