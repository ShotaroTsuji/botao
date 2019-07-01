use botao::fields::enum_fields;
use botao::fields::format_fields;
use botao::fields::format_fields_with;

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

    println!("{}", format_fields(b' ', &["abc", "de", "fgh"]));
    println!("{}", format_fields_with(b' ', &[0u32, 1, 2, 3, 4], u32::to_string));
    println!("{}", format_fields_with(b',', &[0u32, 1, 2, 3, 4], u32::to_string));
}
