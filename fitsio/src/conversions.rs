use super::types::DataType;

pub fn typechar_to_data_type<T: AsRef<str>>(typechar: T) -> DataType {
    match typechar.as_ref() {
        "X" => DataType::TBIT,
        "B" => DataType::TBYTE,
        "L" => DataType::TLOGICAL,
        "A" => DataType::TSTRING,
        "I" => DataType::TSHORT,
        "J" => DataType::TLONG,
        "E" => DataType::TFLOAT,
        "D" => DataType::TDOUBLE,
        "C" => DataType::TCOMPLEX,
        "M" => DataType::TDBLCOMPLEX,
        "K" => DataType::TLONGLONG,
        other => panic!("Unhandled case: {}", other),
    }
}
