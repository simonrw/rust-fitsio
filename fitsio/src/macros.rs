/// Macro to return a fits error if the fits file is not open in readwrite mode
macro_rules! fits_check_readwrite {
    ($fitsfile:expr) => {
        use $crate::errors::FitsError;
        if let Ok($crate::fitsfile::FileOpenMode::READONLY) = $fitsfile.open_mode() {
            return Err(FitsError {
                status: 602,
                message: "cannot alter readonly file".to_string(),
            }
            .into());
        }
    };
}

#[cfg(test)]
/// Macro to allow testing by matching to a pattern.
///
/// Not called simply `assert_matches`, as a macro of that name
/// is in the Rust standard library already
/// (though experimentally as of time of authoring).
macro_rules! assert_matches1 {
    ($value:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {
        {
            let x = $value;
            if !core::matches!(x, $pattern $(if $guard)?) {
                panic!(
                    "pattern-match assert failed\n    value: ({}) == {:?}\n    pattern: {}",
                    core::stringify!($value),
                    x,
                    core::stringify!($pattern $(if $guard)?),
                );
            }
        }
    }
}
