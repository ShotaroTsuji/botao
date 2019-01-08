use botao::text::{DataRecordReader, DataBlockReader};
use std::io::Cursor;

#[test]
fn test_blockreader1() {
    let data =
"0, 0, 0
1, 1, 1
2, 2, 2";
    println!("### Input data");
    println!("{}", data);
    println!("###");

    let data: Vec<u8> = data.into();
    let buf = Cursor::new(data);
    let rdr = DataRecordReader::new(buf);
    let mut rdr = DataBlockReader::<i64, _>::new(rdr);
    let block = rdr.next_block().unwrap().unwrap();
    println!("{:?}", block);
    assert_eq!(block, vec![vec![0, 0, 0], vec![1, 1, 1], vec![2, 2, 2]]);
}

#[test]
fn test_blockreader2() {
    let data =
"0, 0, 0
1, 1, 1
2, 2, 2

3,3,3
4,    4  , 4
5, 5 , 5


6, 6, 6   
7, 7, 7
8, 8, 8
";
    println!("### Input data");
    println!("{}", data);
    println!("###");

    let data: Vec<u8> = data.into();
    let buf = Cursor::new(data);
    let rdr = DataRecordReader::new(buf);
    let mut rdr = DataBlockReader::<i64, _>::new(rdr);
    let block = rdr.next_block().unwrap().unwrap();
    let count = rdr.consume_blanks().unwrap();
    println!("{:?}", block);
    println!("## count = {}", count);
    assert_eq!(block, vec![vec![0; 3], vec![1; 3], vec![2; 3]]);
    assert_eq!(count, 1);

    let block = rdr.next_block().unwrap().unwrap();
    let count = rdr.consume_blanks().unwrap();
    println!("{:?}", block);
    println!("## count = {}", count);
    assert_eq!(block, vec![vec![3; 3], vec![4; 3], vec![5; 3]]);
    assert_eq!(count, 2);

    let block = rdr.next_block().unwrap().unwrap();
    let count = rdr.consume_blanks().unwrap();
    println!("{:?}", block);
    println!("## count = {}", count);
    assert_eq!(block, vec![vec![6; 3], vec![7; 3], vec![8; 3]]);
    assert_eq!(count, 0);

    let block = rdr.next_block().unwrap();
    println!("{:?}", block);
    assert_eq!(block, None);
}
