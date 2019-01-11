use memchr::memchr;
use failure::Fail;
use std::marker::PhantomData;
use std::io;

/// Finds the next field separated by `delim` in the given record.
///
/// This function returns two slices of the type `&str`.
/// The former slice contains the next field and
/// the latter slice contains the remaining data in the record.
/// The whitespaces around records are trimmed.
///
/// # Examples
///
/// ```
/// use botao::text::next_field;
/// let result = next_field(b',', "10, 20, 30");
/// assert_eq!(result, ("10", " 20, 30"));
/// ```
pub fn next_field(delim: u8, record: &str) -> (&str, &str) {
    let record = record.trim();
    if let Some(pos) = memchr(delim, record.as_bytes()) {
        (&record[0..pos].trim(), &record[pos+1..])
    } else {
        (record, "")
    }
}

/// Creates an iterator that returns fields in the given record.
///
/// This function creates an iterator that iterates over the fields.
/// in the given record separated by `delim`.
/// The whitespaces around records are trimmed.
///
/// # Examples
///
/// ```
/// use botao::text::enum_fields;
/// let mut iter = enum_fields(b',', "10, 20, 30");
/// assert_eq!(iter.next(), Some("10"));
/// assert_eq!(iter.next(), Some("20"));
/// assert_eq!(iter.next(), Some("30"));
/// assert_eq!(iter.next(), None);
/// ```
pub fn enum_fields<'a>(delim: u8, record: &'a str) -> EnumFields<'a> {
    EnumFields {
        delim: delim,
        record: record.trim(),
        _phantom: PhantomData,
    }
}

/// An interator type created by the function `enum_fields`.
///
/// See the documentation of the function [`enum_fields`](./fn.enum_fields.html).
pub struct EnumFields<'a> {
    delim: u8,
    record: &'a str,
    _phantom: PhantomData<&'a str>,
}

impl<'a> Iterator for EnumFields<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.record.len() == 0 {
            None
        } else {
            let (field, result) = next_field(self.delim, self.record);
            self.record = result;
            Some(field)
        }
    }
}

#[derive(Debug)]
pub struct DataRecordReaderBuilder<RdType, FdType> {
    record_delimiter: RdType,
    field_delimiter: FdType,
}

impl DataRecordReaderBuilder<(), ()> {
    pub fn new() -> Self {
        DataRecordReaderBuilder {
            record_delimiter: (),
            field_delimiter: (),
        }
    }
}

impl DataRecordReaderBuilder<u8, u8> {
    pub fn build<R: io::BufRead>(self, stream: R) -> DataRecordReader<R> {
        DataRecordReader {
            stream: stream,
            record_delimiter: self.record_delimiter,
            field_delimiter: self.field_delimiter,
            buffer: Vec::new(),
            peek_buf: None,
        }
    }
}

impl<RdType, FdType> DataRecordReaderBuilder<RdType, FdType> {
    pub fn record_delimiter(self, delim: u8) -> DataRecordReaderBuilder<u8, FdType> {
        DataRecordReaderBuilder {
            record_delimiter: delim,
            field_delimiter: self.field_delimiter,
        }
    }

    pub fn field_delimiter(self, delim: u8) -> DataRecordReaderBuilder<RdType, u8> {
        DataRecordReaderBuilder {
            record_delimiter: self.record_delimiter,
            field_delimiter: delim,
        }
    }
}

/// Error type returned by `DataRecordReader` and `DataBlockReader`.
#[derive(Debug, Fail)]
pub enum ReaderError {
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "From UTF-8 error: {}", _0)]
    FromUTF8(#[cause] std::string::FromUtf8Error),
}

/// `DataRecordReader` provides a function of reading records.
///
/// The struct `DataRecordReader` reads the text records in the given file.
/// Each record is stored in each line.
/// The lines are delimited with the LF.
#[derive(Debug)]
pub struct DataRecordReader<R: io::BufRead> {
    stream: R,
    record_delimiter: u8,
    field_delimiter: u8,
    buffer: Vec<u8>,
    peek_buf: Option<DataRecord>,
}

/// `DataRecord` type represents the records in data files.
#[derive(Debug)]
pub enum DataRecord {
    /// A record with fields.
    Fields(Vec<String>),
    /// A comment line.
    Comment(String),
    /// A blank line.
    Blank,
    /// The End-Of-File.
    EOF,
}

impl<R: io::BufRead> DataRecordReader<R> {
    /// Creates a new `DataRecordReader`.
    ///
    /// The method `new` creates a `DataRecordReader` from a stream reader that
    /// implements the trait `std::io::BufRead`.
    pub fn new(stream: R) -> Self {
        DataRecordReader {
            stream: stream,
            record_delimiter: b'\n',
            field_delimiter: b',',
            buffer: Vec::new(),
            peek_buf: None,
        }
    }

    pub fn field_delimiter(&self) -> &u8 {
        &self.field_delimiter
    }

    pub fn set_field_delimiter(&mut self, delim: u8) {
        self.field_delimiter = delim;
    }

    pub fn peek_record(&mut self) -> Result<&DataRecord, failure::Error> {
        if self.peek_buf.is_none() {
            let record = self.next_record()?;
            let _ = self.peek_buf.replace(record);
        }
        Ok(self.peek_buf.as_ref().unwrap())
    }

    pub fn next_record(&mut self) -> Result<DataRecord, failure::Error> {
        if let Some(record) = self.peek_buf.take() {
            return Ok(record);
        }
        let result = self.stream.read_until(self.record_delimiter, &mut self.buffer)
                         .map_err(|e| ReaderError::Io(e))?;
        if result == 0 {
            Ok(DataRecord::EOF)
        } else {
            if self.buffer[0] == b'#' {
                let comment = String::from_utf8(self.buffer.clone())
                                     .map_err(|e| ReaderError::FromUTF8(e))?;
                self.buffer.clear();
                Ok(DataRecord::Comment(comment))
            } else {
                let s = String::from_utf8(self.buffer.clone())
                               .map_err(|e| ReaderError::FromUTF8(e))?;
                self.buffer.clear();
                let fields: Vec<String>
                    = enum_fields(self.field_delimiter, s.as_str()).map(|s| s.to_owned()).collect();
                if fields.len() == 0 {
                    Ok(DataRecord::Blank)
                } else {
                    Ok(DataRecord::Fields(fields))
                }
            }
        }
    }
}

/// A reader type that reads data blocks in the given file.
///
/// The type `DataBlockReader` is built on `DataRecordReader`.
/// A data block is a contiguous series of records.
/// The blocks are separated by one or more blank lines.
#[derive(Debug)]
pub struct DataBlockReader<T, R>
where
    R: io::BufRead,
    T: std::str::FromStr
{
    reader: DataRecordReader<R>,
    _phantom: PhantomData<fn() -> T>,
}

impl<T, R> DataBlockReader<T, R>
where
    R: io::BufRead,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: failure::Fail,
{
    pub fn new(reader: DataRecordReader<R>) -> Self {
        DataBlockReader {
            reader: reader,
            _phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> DataRecordReader<R> {
        self.reader
    }

    /// Returns a next block or `None`.
    ///
    /// This method reads a data block. It reads only one separating blank line.
    /// If there may be multiple blank lines, you must call the method `consume_blanks`
    /// after calling this method.
    pub fn next_block(&mut self) -> Result<Option<Vec<Vec<T>>>, failure::Error> {
        let mut block: Option<Vec<Vec<T>>> = None;
        loop {
            let record = self.reader.peek_record()?;
            match record {
                DataRecord::EOF | DataRecord::Blank => break,
                _ => {},
            };
            let record = self.reader.next_record()?;
            match record {
                DataRecord::Comment(_) => continue,
                DataRecord::Fields(fields) => {
                    let vec = fields.iter().map(|f| T::from_str(f))
                                           .collect::<Result<Vec<T>, _>>();
                    match vec {
                        Ok(vec) => { block.get_or_insert_with(|| Vec::new()).push(vec); },
                        Err(e) => { return Err(e.into()); },
                    };
                },
                _ => panic!("unreachable!"),
            };
        };
        Ok(block)
    }

    /// Consumes blank lines and returns the count of the consumed blank lines.
    pub fn consume_blanks(&mut self) -> Result<usize, failure::Error> {
        let mut count = 0;
        loop {
            let record = self.reader.peek_record()?;
            match record {
                DataRecord::Blank | DataRecord::Comment(_) => {
                    count += 1;
                    self.reader.next_record().unwrap();
                },
                _ => break,
            };
        };
        Ok(count)
    }
}
