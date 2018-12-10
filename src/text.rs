extern crate failure;
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

impl From<::std::string::FromUtf8Error> for Error {
    fn from(error: ::std::string::FromUtf8Error) -> Error {
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

    pub fn next_record(&mut self) -> Result<DataRecord, Error> {
        let mut buf = Vec::new();
        let result = self.buffer.read_until(self.record_delimiter, &mut buf)?;
        if result == 0 {
            Ok(DataRecord::EOF)
        } else {
            if buf[0] == b'#' {
                Ok(DataRecord::Comment(String::from_utf8(buf)?))
            } else {
                let s = String::from_utf8(buf)?;
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
