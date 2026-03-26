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

/// A trait to associate a type with the appropriate [`DataType`].
///
/// This helper trait is useful as the different values of [`DataType`],
/// as used by CFITSIO, refer to **C** types, not **Rust** types.
/// Some compile-time calculations are used to ensure the right C type
/// is associated with the Rust type.
pub(crate) trait HasFitsDataType {
    /// The [`DataType`] associated with `Self`.
    const FITS_DATA_TYPE: DataType;
}

/// Convenience function; returns the size and alignment of `T`.
const fn size_align<T: Sized>() -> (usize, usize) {
    (core::mem::size_of::<T>(), core::mem::align_of::<T>())
}

/// Macro to compute the signedness, size, and alignment of a type.
///
/// `$name` will be the constant `(is $t unsigned, (size of $t, alignment of $t))`.
macro_rules! ctype_sign_sz_align {
    ($($t:ty, $name:ident);* $(;)?) => {
        $(
            // we use $t::MIN == 0 here and in has_fits_data_type_int! to distinguish between signed and unsigned types
            const $name: (bool, (usize, usize)) = (<$t>::MIN == 0, size_align::<$t>());
        )*
    };
}

ctype_sign_sz_align!(
    core::ffi::c_schar,  SCHAR;
    core::ffi::c_uchar,  UCHAR;
    core::ffi::c_short,  SHORT;
    core::ffi::c_ushort, USHORT;

    core::ffi::c_int,    INT;
    core::ffi::c_uint,   UINT;
    core::ffi::c_long,   LONG;
    core::ffi::c_ulong,  ULONG;

    core::ffi::c_longlong,  LONGLONG;
    core::ffi::c_ulonglong, ULONGLONG;
);

/// Generates an implementation of [`HasFitsDataType`] for integer type `$t`,
/// making sure we match `$t` with an equivalent C type.
macro_rules! has_fits_data_type_int {
    ($t:ty) => {
        impl HasFitsDataType for $t {
            const FITS_DATA_TYPE: DataType = {
                // we use $t::MIN == 0 here and in ctype_sign_sz_align! to distinguish between signed and unsigned types

                #[allow(unreachable_patterns)]
                match (<$t>::MIN == 0, size_align::<$t>()) {
                    UINT => DataType::TUINT,
                    USHORT => DataType::TUSHORT,
                    UCHAR => DataType::TBYTE,
                    ULONG => DataType::TULONG,
                    ULONGLONG => DataType::TULONGLONG,

                    INT => DataType::TINT,
                    SHORT => DataType::TSHORT,
                    SCHAR => DataType::TSBYTE,
                    LONG => DataType::TLONG,
                    LONGLONG => DataType::TLONGLONG,

                    _ => panic!(concat!(
                        "Type ",
                        stringify!($t),
                        " does not have a corresponding DataType"
                    )),
                }
            };
        }
    };
}

has_fits_data_type_int!(u8);
has_fits_data_type_int!(u16);
has_fits_data_type_int!(u32);
has_fits_data_type_int!(u64);
has_fits_data_type_int!(i8);
has_fits_data_type_int!(i16);
has_fits_data_type_int!(i32);
has_fits_data_type_int!(i64);

/// Generates an implementation of [`HasFitsDataType`] for floating-point type `$t`,
/// making sure we match `$t` with an equivalent C type.
macro_rules! has_fits_data_type_floating {
    ($t:ty) => {
        impl HasFitsDataType for $t {
            const FITS_DATA_TYPE: DataType = {
                use core::ffi::{c_double, c_float};

                const FLOAT: (usize, usize) = size_align::<c_float>();
                const DOUBLE: (usize, usize) = size_align::<c_double>();

                #[allow(unreachable_patterns)]
                match size_align::<$t>() {
                    DOUBLE => DataType::TDOUBLE,
                    FLOAT => DataType::TFLOAT,
                    _ => panic!(concat!(
                        "Type ",
                        stringify!($t),
                        " does not have a corresponding DataType"
                    )),
                }
            };
        }
    };
}

has_fits_data_type_floating!(f32);
has_fits_data_type_floating!(f64);

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
