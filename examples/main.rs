extern crate botao;
use botao::text::enum_fields;
use botao::text::{DataRecordReader, DataRecord, DataBlockReader};
use std::io;
use std::fs::File;

fn test_enum_fields(delim: u8, record: &str) {
    println!("delim : {:?}", delim as char);
    println!("record: {:?}", record);
    for field in enum_fields(delim, record) {
        println!("FIELD: {:?}", field);
    }
}

fn test_nested(delim: u8, record: &str) {
    println!("NESTED");
    println!("record: {:?}", record);
    for field in enum_fields(b';', record) {
        println!("FIELD");
        for field in enum_fields(delim, field) {
            println!("{:?}", field);
        }
    }
}

fn test_datarecord_reader<R>(buf: R)
where
    R: io::BufRead + ::std::fmt::Debug,
{
    let mut rdr = DataRecordReader::new(buf);
    println!("# test_datarecord_reader");
    println!("{:?}", rdr);
    loop {
        let record = rdr.next_record().unwrap();
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

fn test_datarecord_peek<R>(buf: R)
where
    R: io::BufRead + ::std::fmt::Debug,
{
    let mut rdr = DataRecordReader::new(buf);
    println!("# test_datarecord_reader");
    println!("{:?}", rdr);
    loop {
        let record = rdr.peek_record().unwrap();
        println!("1st : {:?}", record);
        let record = rdr.peek_record().unwrap();
        println!("2nd : {:?}", record);
        let record = rdr.next_record().unwrap();
        println!("Last: {:?}", record);
        match record {
            DataRecord::EOF => { break; },
            _ => {},
        }
    }
}

fn test_datablock_reader<R>(buf: R)
where
    R: io::BufRead + ::std::fmt::Debug,
{
    let rdr = DataRecordReader::new(buf);
    let mut rdr = DataBlockReader::<f64, _>::new(rdr);
    println!("# test_datablock_reader");
    println!("{:?}", rdr);

    while let Some(vec) = rdr.next_block().unwrap() {
        println!("{:?}", vec);
        let count = rdr.consume_blanks().unwrap();
        println!("... {} blank lines are consumed.", count);
    }
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
    test_datarecord_reader(io::BufReader::new(&data[..]));

    println!("");
    test_datarecord_reader(io::BufReader::new(File::open("examples/test1.txt").unwrap()));

    println!("");
    test_datarecord_reader(io::BufReader::new(File::open("examples/test2.txt").unwrap()));

    println!("");
    test_datarecord_peek(io::BufReader::new(File::open("examples/test2.txt").unwrap()));

    println!("");
    test_datablock_reader(io::BufReader::new(File::open("examples/test1.txt").unwrap()));

    println!("");
    test_datablock_reader(io::BufReader::new(File::open("examples/test2.txt").unwrap()));

    println!("");
    test_datablock_reader(io::BufReader::new(File::open("examples/test3.txt").unwrap()));

    println!("");
    test_datablock_reader(io::BufReader::new(File::open("examples/test4.txt").unwrap()));
}
