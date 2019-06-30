use memchr::memchr;
use failure::Fail;
use std::fmt::Display;
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
/// use botao::fields::next_field;
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
/// use botao::fields::enum_fields;
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

/// Formats a record into string.
///
/// # Arguments
/// * `delim` - a field delimiter
/// * `record` - a record or fields
///
/// # Examples
/// ```
/// use botao::fields::format_fields;
/// let s = format_fields(b' ', &[0, 1, 2, 3]);
/// assert_eq!(s, "0 1 2 3");
/// let s = format_fields(b',', &[0, 1, 2, 3]);
/// assert_eq!(s, "0,1,2,3");
/// ```
pub fn format_fields<T: Display>(delim: u8, record: &[T]) -> String {
    let delim: char = delim.into();
    let len = record.len();
    let mut s = String::new();

    for i in 0..(len-1) {
        let t = record[i].to_string();
        s.push_str(&t);
        s.push(delim);
    }
    let t = record[len-1].to_string();
    s.push_str(&t);

    s
}
