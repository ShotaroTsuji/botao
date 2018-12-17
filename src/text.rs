use memchr::memchr;
use failure::Fail;
use std::marker::PhantomData;
use std::io;

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
pub enum ReaderError {
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "From UTF-8 error: {}", _0)]
    FromUTF8(#[cause] std::string::FromUtf8Error),
    #[fail(display = "Data table size error")]
    SizeError,
}

#[derive(Debug)]
pub struct DataRecordReader<R: io::BufRead> {
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

impl<R: io::BufRead> DataRecordReader<R> {
    pub fn new(buffer: R) -> Self {
        DataRecordReader {
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

    pub fn next_record(&mut self, buf: &mut Vec<u8>) -> Result<DataRecord, failure::Error> {
        let result = self.buffer.read_until(self.record_delimiter, buf)
                         .map_err(|e| ReaderError::Io(e))?;
        if result == 0 {
            Ok(DataRecord::EOF)
        } else {
            if buf[0] == b'#' {
                let comment = String::from_utf8(buf.clone())
                                     .map_err(|e| ReaderError::FromUTF8(e))?;
                buf.clear();
                Ok(DataRecord::Comment(comment))
            } else {
                let s = String::from_utf8(buf.clone())
                               .map_err(|e| ReaderError::FromUTF8(e))?;
                buf.clear();
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
pub struct DataBlockReader<T, R>
where
    R: io::BufRead,
    T: std::str::FromStr
{
    reader: DataRecordReader<R>,
    _phantom: PhantomData<fn() -> T>,
}

// blank lines are treated as missing values.
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

    pub fn next_record(&mut self, buf: &mut Vec<u8>) -> Result<Option<Vec<T>>, failure::Error> {
        let record = self.reader.next_record(buf)?;
        match record {
            DataRecord::EOF => Ok(None),
            DataRecord::Comment(_) => {
                self.next_record(buf)
            }
            DataRecord::Blank => Ok(Some(Vec::new())),
            DataRecord::Fields(fields) => {
                let vec = fields.iter()
                                .map(|f| T::from_str(f))
                                .collect::<Result<Vec<_>, _>>();
                match vec {
                    Ok(vec) => Ok(Some(vec)),
                    Err(e) => Err(e.into()),
                }
            },
        }
    }

    pub fn into_table_with_size_check(mut self) -> Result<Vec<Vec<T>>, failure::Error> {
        let mut result = Vec::new();
        let mut buf = Vec::new();

        let mut _field_len = None;
        let record = self.next_record(&mut buf)?;

        if let Some(vec) = record {
            _field_len = Some(vec.len());
            result.push(vec);
        } else {
            return Ok(result);
        }

        let field_len = _field_len.unwrap();

        while let Some(vec) = self.next_record(&mut buf)? {
            if vec.len() != field_len {
                return Err(ReaderError::SizeError.into());
            }
            result.push(vec);
        }

        Ok(result)
    }
}

/*
#[derive(Debug)]
pub struct DataBlocksReader<T, R>
where
    R: io::BufRead,
    T: std::str::FromStr
{
    reader: DataFileReader<R>,
    _phantom: PhantomData<fn() -> T>,
}

// blank lines are treated as missing values.
impl<T, R> DataBlocksReader<T, R>
where
    R: io::BufRead,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: failure::Fail,
{
    pub fn new(reader: DataFileReader<R>) -> Self {
        DataBlocksReader {
            reader: reader,
            _phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> DataFileReader<R> {
        self.reader
    }

    pub fn next_block(&mut self, buf: &mut Vec<u8>) -> Option<Vec<Vec<T>>> {
        enum State {
            None,
            Fields,
            Blank,
        }
        let mut prev_state = State::None;
        let mut result: Option<Vec<Vec<T>>> = None;

        loop {
            let record = self.reader.next_record(buf).unwrap();
            match record {
                DataRecord::EOF => { break; },
                DataRecord::Comment(_) => { continue; },
                DataRecord::Fields(fields) => {
                    let vec = fields.iter().map(|f| T::from_str(f).unwrap())
                                    .collect::<Vec<T>>();
                    result.get_or_insert_with(|| Vec::new()).push(vec);
                    prev_state = State::Fields;
                },
                DataRecord::Blank => {
                    match prev_state {
                        State::Blank => { break; },
                        _ => { prev_state = State::Blank; },
                    };
                },
            };
        }

        result
    }
}
*/
