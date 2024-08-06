//! Data types used within `fitsio`

/// Enumeration of different data types used for column and key types
#[allow(missing_docs, clippy::upper_case_acronyms)]
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    TBIT,
    TBYTE,
    TSBYTE,
    TLOGICAL,
    TSTRING,
    TUSHORT,
    TSHORT,
    TUINT,
    TINT,
    TULONG,
    TLONG,
    TFLOAT,
    TULONGLONG,
    TLONGLONG,
    TDOUBLE,
    TCOMPLEX,
    TDBLCOMPLEX,
}

#[cfg(test)]
mod test {
    use crate::fitsfile::{CaseSensitivity, FileOpenMode};
    use crate::hdu::HduInfo;
    use crate::images::ImageType;
    use crate::types::DataType;

    #[test]
    fn test_image_types() {
        assert_eq!(i8::from(ImageType::UnsignedByte), 8);
        assert_eq!(i8::from(ImageType::Byte), 10);
        assert_eq!(i8::from(ImageType::Short), 16);
        assert_eq!(i8::from(ImageType::UnsignedShort), 20);
        assert_eq!(i8::from(ImageType::Long), 32);
        assert_eq!(i8::from(ImageType::LongLong), 64);
        assert_eq!(i8::from(ImageType::Float), -32);
        assert_eq!(i8::from(ImageType::Double), -64);
    }

    #[test]
    fn test_hdu_types() {
        let image_info = HduInfo::ImageInfo {
            shape: Vec::new(),
            image_type: ImageType::LongLong,
        };

        let table_info = HduInfo::TableInfo {
            column_descriptions: Vec::new(),
            num_rows: 0,
        };

        assert_eq!(i32::from(image_info), 0);
        assert_eq!(i32::from(table_info), 2);
        assert_eq!(i32::from(HduInfo::AnyInfo), -1);
    }

    #[test]
    fn test_file_open_modes() {
        assert_eq!(u8::from(FileOpenMode::READONLY), 0);
        assert_eq!(u8::from(FileOpenMode::READWRITE), 1);
    }

    #[test]
    fn test_case_sensitivity() {
        assert_eq!(u8::from(CaseSensitivity::CASESEN), 1);
        assert_eq!(u8::from(CaseSensitivity::CASEINSEN), 0);
    }

    #[test]
    fn test_converting_from_data_type() {
        assert_eq!(u8::from(DataType::TBIT), 1);
        assert_eq!(u8::from(DataType::TBYTE), 11);
        assert_eq!(u8::from(DataType::TLOGICAL), 14);
        assert_eq!(u8::from(DataType::TSTRING), 16);
        assert_eq!(u8::from(DataType::TSHORT), 21);
        assert_eq!(u8::from(DataType::TLONG), 41);
        assert_eq!(u8::from(DataType::TFLOAT), 42);
        assert_eq!(u8::from(DataType::TDOUBLE), 82);
    }
}
