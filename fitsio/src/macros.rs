/// Macro to return a fits error if the fits file is not open in readwrite mode
macro_rules! fits_check_readwrite {
    ($fitsfile:expr) => {
        use $crate::errors::FitsError;
        if let Ok($crate::fitsfile::FileOpenMode::READONLY) = $fitsfile.open_mode() {
            return Err(FitsError {
                status: 602,
                message: "cannot alter readonly file".to_string(),
            }.into());
        }
    };
}
