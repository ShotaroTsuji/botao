use memchr::memchr;
use failure::Fail;
use std::marker::PhantomData;
use std::io;
use std::fmt;

pub fn next_field(delim: u8, record: &str) -> (&str, &str) {
    let record = record.trim();
    if let Some(pos) = memchr(delim, record.as_bytes()) {
        (&record[0..pos].trim(), &record[pos+1..])
    } else {
        (record, "")
    }
}

pub fn enum_fields<'a>(delim: u8, record: &'a str) -> EnumFields<'a> {
    EnumFields {
        delim: delim,
        record: record.trim(),
        _phantom: PhantomData,
    }
}

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

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "From UTF-8 error")]
    FromUTF8,
    #[fail(display = "Attribute parse error")]
    Attribute,
}

#[derive(Debug)]
pub struct Error {
    inner: failure::Context<ErrorKind>,
}

impl failure::Fail for Error {
    fn cause(&self) -> Option<&failure::Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn new(inner: failure::Context<ErrorKind>) -> Error {
        Error { inner }
    }

    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: failure::Context::new(kind),
        }
    }
}

impl From<failure::Context<ErrorKind>> for Error {
    fn from(inner: failure::Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error {
            inner: error.context(ErrorKind::Io),
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Error {
        Error {
            inner: error.context(ErrorKind::FromUTF8),
        }
    }
}

#[derive(Debug)]
pub struct DataFileReader<R: io::BufRead> {
    buffer: R,
    record_delimiter: u8,
    field_delimiter: u8,
}

#[derive(Debug)]
pub enum DataRecord {
    Fields(Vec<String>),
    Comment(String),
    Blank,
    EOF,
}

impl<R: io::BufRead> DataFileReader<R> {
    pub fn new(buffer: R) -> Self {
        DataFileReader {
            buffer: buffer,
            record_delimiter: b'\n',
            field_delimiter: b',',
        }
    }

    pub fn field_delimiter(&self) -> &u8 {
        &self.field_delimiter
    }

    pub fn set_field_delimiter(&mut self, delim: u8) {
        self.field_delimiter = delim;
    }

    pub fn next_record(&mut self, buf: &mut Vec<u8>) -> Result<DataRecord, Error> {
        let result = self.buffer.read_until(self.record_delimiter, buf)?;
        if result == 0 {
            Ok(DataRecord::EOF)
        } else {
            if buf[0] == b'#' {
                Ok(DataRecord::Comment(String::from_utf8(buf.clone())?))
            } else {
                let s = String::from_utf8(buf.clone())?;
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

#[derive(Debug)]
pub struct DataTableReader<T, R>
where
    R: io::BufRead,
    T: std::str::FromStr
{
    reader: DataFileReader<R>,
    _phantom: PhantomData<fn() -> T>,
}

// blank lines are treated as missing values.
impl<T, R> DataTableReader<T, R>
where
    R: io::BufRead,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    pub fn new(reader: DataFileReader<R>) -> Self {
        DataTableReader {
            reader: reader,
            _phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> DataFileReader<R> {
        self.reader
    }

    pub fn next_record(&mut self, buf: &mut Vec<u8>) -> Option<Vec<T>> {
        let record = self.reader.next_record(buf).unwrap();
        match record {
            DataRecord::EOF => None,
            DataRecord::Comment(_) => {
                buf.clear();
                self.next_record(buf)
            }
            DataRecord::Blank => Some(Vec::new()),
            DataRecord::Fields(fields) => {
                Some(fields.iter().map(|f| T::from_str(f).unwrap()).collect())
            },
        }
    }

    pub fn into_table_with_size_check(mut self) -> Vec<Vec<T>> {
        let mut result = Vec::new();
        let mut buf = Vec::new();

        let mut field_len = None;
        let record = self.next_record(&mut buf);

        if let Some(vec) = record {
            field_len = Some(vec.len());
            result.push(vec);
        } else {
            return result;
        }

        let field_len = field_len.unwrap();
        buf.clear();

        while let Some(vec) = self.next_record(&mut buf) {
            if vec.len() != field_len {
                panic!("size errror");
            }
            result.push(vec);
            buf.clear();
        }

        result
    }
}
