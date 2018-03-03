//! Data types used within `fitsio`
use columndescription::*;

/// Enumeration of different data types used for column and key types
#[allow(missing_docs)]
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
    TLONGLONG,
    TFLOAT,
    TDOUBLE,
    TCOMPLEX,
    TDBLCOMPLEX,
}

macro_rules! datatype_into_impl {
    ($t: ty) => (
        impl From<DataType> for $t {
            fn from(original: DataType) -> $t {
                match original {
                    DataType::TBIT => 1,
                    DataType::TBYTE => 11,
                    DataType::TSBYTE => 12,
                    DataType::TLOGICAL => 14,
                    DataType::TSTRING => 16,
                    DataType::TUSHORT => 20,
                    DataType::TSHORT => 21,
                    DataType::TUINT => 30,
                    DataType::TINT => 31,
                    DataType::TULONG => 40,
                    DataType::TLONG => 41,
                    DataType::TLONGLONG => 81,
                    DataType::TFLOAT => 42,
                    DataType::TDOUBLE => 82,
                    DataType::TCOMPLEX => 83,
                    DataType::TDBLCOMPLEX => 163,
                }
            }
        }
    )
}

datatype_into_impl!(u8);
datatype_into_impl!(i32);
datatype_into_impl!(u32);
datatype_into_impl!(i64);
datatype_into_impl!(u64);

/// Data types used for defining images
#[allow(non_camel_case_types, missing_docs)]
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImageType {
    BYTE_IMG,
    SBYTE_IMG,
    SHORT_IMG,
    USHORT_IMG,
    LONG_IMG,
    ULONG_IMG,
    LONGLONG_IMG,
    FLOAT_IMG,
    DOUBLE_IMG,
}

macro_rules! imagetype_into_impl {
    ($t: ty) => (
        impl From<ImageType> for $t {
            fn from(original: ImageType) -> $t {
                match original {
                    ImageType::BYTE_IMG => 8,
                    ImageType::SBYTE_IMG => 10,
                    ImageType::SHORT_IMG => 16,
                    ImageType::USHORT_IMG => 20,
                    ImageType::LONG_IMG => 32,
                    ImageType::ULONG_IMG => 40,
                    ImageType::LONGLONG_IMG => 64,
                    ImageType::FLOAT_IMG => -32,
                    ImageType::DOUBLE_IMG => -64,
                }
            }
        }
    )
}

imagetype_into_impl!(i8);
imagetype_into_impl!(i32);
imagetype_into_impl!(i64);

/// Description of the current HDU
///
/// If the current HDU is an image, then
/// [`fetch_hdu_info`](struct.FitsFile.html#method.fetch_hdu_info) returns `HduInfo::ImageInfo`.
/// Otherwise the variant is `HduInfo::TableInfo`.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum HduInfo {
    ImageInfo {
        shape: Vec<usize>,
        image_type: ImageType,
    },
    TableInfo {
        column_descriptions: Vec<ConcreteColumnDescription>,
        num_rows: usize,
    },
    AnyInfo,
}

macro_rules! hduinfo_into_impl {
    ($t: ty) => (
        impl From<HduInfo> for $t {
            fn from(original: HduInfo) -> $t {
                match original {
                    HduInfo::ImageInfo { .. } => 0,
                    HduInfo::TableInfo { .. } => 2,
                    HduInfo::AnyInfo => -1,
                }
            }
        }
    )
}

hduinfo_into_impl!(i8);
hduinfo_into_impl!(i32);
hduinfo_into_impl!(i64);

/// Enumeration of file open modes
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum FileOpenMode {
    READONLY,
    READWRITE,
}

macro_rules! fileopenmode_into_impl {
    ($t: ty) => (
        impl From<FileOpenMode> for $t {
            fn from(original: FileOpenMode) -> $t {
                match original {
                    FileOpenMode::READONLY => 0,
                    FileOpenMode::READWRITE => 1,
                }
            }
        }
        )
}

fileopenmode_into_impl!(u8);
fileopenmode_into_impl!(u32);
fileopenmode_into_impl!(u64);
fileopenmode_into_impl!(i8);
fileopenmode_into_impl!(i32);
fileopenmode_into_impl!(i64);

/// Enumeration of options for case sensitivity
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum CaseSensitivity {
    CASEINSEN,
    CASESEN,
}

macro_rules! casesensitivity_into_impl {
    ($t: ty) => (
        impl From<CaseSensitivity> for $t {
            fn from(original: CaseSensitivity) -> $t {
                match original {
                    CaseSensitivity::CASEINSEN => 0,
                    CaseSensitivity::CASESEN => 1,
                }
            }
        }
        )
}

casesensitivity_into_impl!(u8);
casesensitivity_into_impl!(u32);
casesensitivity_into_impl!(u64);
casesensitivity_into_impl!(i8);
casesensitivity_into_impl!(i32);
casesensitivity_into_impl!(i64);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_image_types() {
        assert_eq!(i8::from(ImageType::BYTE_IMG), 8);
        assert_eq!(i8::from(ImageType::SBYTE_IMG), 10);
        assert_eq!(i8::from(ImageType::SHORT_IMG), 16);
        assert_eq!(i8::from(ImageType::USHORT_IMG), 20);
        assert_eq!(i8::from(ImageType::LONG_IMG), 32);
        assert_eq!(i8::from(ImageType::LONGLONG_IMG), 64);
        assert_eq!(i8::from(ImageType::FLOAT_IMG), -32);
        assert_eq!(i8::from(ImageType::DOUBLE_IMG), -64);
    }

    #[test]
    fn test_hdu_types() {
        let image_info = HduInfo::ImageInfo {
            shape: Vec::new(),
            image_type: ImageType::LONGLONG_IMG,
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
