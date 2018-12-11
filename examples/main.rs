extern crate botao;
use botao::text::enum_fields;
use botao::text::{DataFileReader, DataRecord, DataTableReader};
use std::io;
use std::fs::File;

fn test_enum_fields(delim: u8, record: &str) {
    println!("delim : '{}'", delim as char);
    println!("record: {}", record);
    for field in enum_fields(delim, record) {
        println!("FIELD: {:?}", field);
    }
}

fn test_nested(delim: u8, record: &str) {
    println!("NESTED");
    println!("record: {}", record);
    for field in enum_fields(b';', record) {
        println!("FIELD");
        for field in enum_fields(delim, field) {
            println!("{}", field);
        }
    }
}

fn test_datafile_reader<R>(buf: R)
where
    R: io::BufRead + ::std::fmt::Debug,
{
    let mut rdr = DataFileReader::new(buf);
    let mut buf = Vec::new();
    println!("# test_datafile_reader");
    println!("{:?}", rdr);
    loop {
        buf.clear();
        let record = rdr.next_record(&mut buf).unwrap();
        match record {
            DataRecord::Fields(fields) => {
                println!("FIELDS: {:?}", fields);
            },
            DataRecord::Comment(comment) => {
                println!("COMMENT: {:?}", comment);
            },
            DataRecord::Blank => {
                println!("BLANK");
            },
            DataRecord::EOF => {
                println!("EOF");
                break;
            },
        }
    }
}

fn test_datatable_reader<R>(buf: R)
where
    R: io::BufRead + ::std::fmt::Debug,
{
    let rdr = DataFileReader::new(buf);
    let mut rdr = DataTableReader::<f64, _>::new(rdr);
    let mut buf: Vec<u8> = Vec::new();
    println!("# test_datatable_reader");
    println!("{:?}", rdr);

    while let Some(vec) = rdr.next_record(&mut buf) {
        println!("{:?}", vec);
        buf.clear();
    }
}

fn test_datatable_into<R>(buf: R)
where
    R: io::BufRead + ::std::fmt::Debug,
{
    let rdr = DataFileReader::new(buf);
    let rdr = DataTableReader::<f64, _>::new(rdr);
    println!("# test_datatable_reader");
    println!("{:?}", rdr);

    let table = rdr.into_table_with_size_check();
    println!("{:?}", table);
}


fn main() {
    test_enum_fields(b',', "10, 20, 30, 40");
    test_enum_fields(b',', "10 , 20  , 30   , 40    ");
    test_enum_fields(b',', "10, 20, 30, 40");
    test_enum_fields(b',', "10 , 20  , 30   , 40    ");
    test_enum_fields(b' ', "10 20  30   40 ");
    test_enum_fields(b' ', "1.2   3.4   2.342 12.23");
    test_enum_fields(b' ', "apple banana   orange");
    test_nested(b',', "10, 20, 30, 40; 3.4");
    test_nested(b',', "10, 20, 30, 40,; 3.4");
    test_nested(b' ', "10 20 30   40  ; 3.4");
    test_enum_fields(b',', "\n");
    test_enum_fields(b' ', "     \n");
    test_enum_fields(b',', ",\n");

    println!("");
    let data = b"10, 20, 30, 40\n5, 6, 7, 8\n\n1.2, 3.4 ,.05, 0.001\n";
    test_datafile_reader(io::BufReader::new(&data[..]));

    println!("");
    test_datafile_reader(io::BufReader::new(File::open("examples/test1.txt").unwrap()));

    println!("");
    test_datafile_reader(io::BufReader::new(File::open("examples/test2.txt").unwrap()));

    println!("");
    test_datatable_reader(io::BufReader::new(File::open("examples/test1.txt").unwrap()));

    println!("");
    test_datatable_reader(io::BufReader::new(File::open("examples/test2.txt").unwrap()));

    println!("");
    test_datatable_into(io::BufReader::new(File::open("examples/test1.txt").unwrap()));

    println!("");
    test_datatable_into(io::BufReader::new(File::open("examples/test2.txt").unwrap()));
}
