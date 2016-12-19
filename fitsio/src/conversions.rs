use super::sys;

pub fn typechar_to_data_type<T: AsRef<str>>(typechar: T) -> sys::DataType {
    match typechar.as_ref() {
        "X" => sys::DataType::TBIT,
        "B" => sys::DataType::TBYTE,
        "L" => sys::DataType::TLOGICAL,
        "A" => sys::DataType::TSTRING,
        "I" => sys::DataType::TSHORT,
        "J" => sys::DataType::TLONG,
        "E" => sys::DataType::TFLOAT,
        "D" => sys::DataType::TDOUBLE,
        "C" => sys::DataType::TCOMPLEX,
        "M" => sys::DataType::TDBLCOMPLEX,
        "K" => sys::DataType::TLONGLONG,
        other => panic!("Unhandled case: {}", other),
    }
}
