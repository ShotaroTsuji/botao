use memchr::memchr;
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

#[derive(Debug)]
pub struct DataFileReader<R: io::BufRead> {
    buffer: R,
    record_delimiter: u8,
    field_delimiter: u8,
}

#[derive(Debug)]
pub enum DataRecord {
    Fields(Vec<String>),
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

    pub fn next_record(&mut self) -> DataRecord {
        let mut buf = Vec::new();
        let result = self.buffer.read_until(self.record_delimiter, &mut buf);
        if result.unwrap() == 0 {
            DataRecord::EOF
        } else {
            let s = String::from_utf8(buf).unwrap();
            let fields: Vec<String>
                = enum_fields(self.field_delimiter, s.as_str()).map(|s| s.to_owned()).collect();
            if fields.len() == 0 {
                DataRecord::Blank
            } else {
                DataRecord::Fields(fields)
            }
        }
    }
}
