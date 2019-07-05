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

/// Formats a record.
///
/// This function is a wrapper for the function `format_fields_from_iter`.
/// It takes a delimiter and an slice of strings.
pub fn format_fields<T: AsRef<str>>(delim: u8, record: &[T]) -> String {
    let mut buf = String::new();
    format_fields_from_iter(delim, record.iter(), &mut buf);
    buf
}

/// Formats a record into a string.
///
/// This function is a wrapper for the function `format_fields_from_iter`.
/// For convenience, it takes a slice and a closure or function that
/// converts its elements into string.
pub fn format_fields_with<T, F>(delim: u8, record: &[T], f: F) -> String
where
    F: Fn(&T) -> String,
{
    let mut buf = String::new();
    format_fields_from_iter(delim, record.iter().map(f), &mut buf);
    buf
}

/// Formats a record into a string with a delimiter.
///
/// # Arguments
/// * `delim` - a delimiter.
/// * `iter` - an iterator that produces fields.
/// * `buf` - a buffer where the formatted string is stored.
///
/// # Examples
/// ```
/// use botao::fields::format_fields_from_iter;
/// let a = [0u32, 1, 2, 3, 4];
/// let mut buf = String::new();
/// format_fields_from_iter(b' ', a.iter().map(u32::to_string), &mut buf);
/// assert_eq!(buf, "0 1 2 3 4");
/// ```
pub fn format_fields_from_iter<T, I>(delim: u8, mut iter: I, buf: &mut String)
where
    T: AsRef<str>,
    I: Iterator<Item=T>,
{
    let delim: char = delim.into();

    match iter.next() {
        Some(field) => {
            buf.push_str(field.as_ref());
            for field in iter {
                buf.push(delim);
                buf.push_str(field.as_ref());
            }
        },
        None => {},
    }
}
