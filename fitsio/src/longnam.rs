//! Long name wrappers of fitsio functions

// Disable clippy warnings as C uses long argument lists
#![allow(clippy::too_many_arguments, clippy::upper_case_acronyms)]
#![allow(unused_imports, dead_code)]

pub(crate) use crate::sys::{
    ffclos, ffcopy, ffcrim, ffcrtb, ffdcol, ffdhdu, ffflmd, ffgbcl, ffgcdw, ffgcno, ffgcvd, ffgcve,
    ffgcvi, ffgcvj, ffgcvjj, ffgcvk, ffgcvs, ffgcvui, ffgcvuj, ffgcvujj, ffgcvuk, ffghdn, ffghdt,
    ffgidm, ffgiet, ffgisz, ffgkyd, ffgkye, ffgkyj, ffgkyjj, ffgkyl, ffgkys, ffgncl, ffgnrw, ffgpv,
    ffgsv, fficol, ffinit, ffmahd, ffmnhd, ffopen, ffpcl, ffpcls, ffphps, ffpky, ffpkyd, ffpkye,
    ffpkyl, ffpkys, ffppr, ffpss, ffrsim, ffthdu, fitsfile, LONGLONG,
};
pub use libc::{
    c_char, c_double, c_float, c_int, c_long, c_short, c_uint, c_ulong, c_ulonglong, c_ushort,
    c_void,
};

pub(crate) unsafe fn fits_close_file(fptr: *mut fitsfile, status: *mut libc::c_int) -> c_int {
    ffclos(fptr, status)
}

pub(crate) unsafe fn fits_copy_hdu(
    infptr: *mut fitsfile,
    outfptr: *mut fitsfile,
    morekeys: c_int,
    status: *mut c_int,
) -> c_int {
    ffcopy(infptr, outfptr, morekeys, status)
}

pub(crate) unsafe fn fits_create_img(
    fptr: *mut fitsfile,
    bitpix: c_int,
    naxis: c_int,
    naxes: *mut c_long,
    status: *mut c_int,
) -> c_int {
    ffcrim(fptr, bitpix, naxis, naxes, status)
}

pub(crate) unsafe fn fits_create_tbl(
    fptr: *mut fitsfile,
    tbltype: c_int,
    naxis2: LONGLONG,
    tfields: c_int,
    ttype: *mut *mut c_char,
    tform: *mut *mut c_char,
    tunit: *mut *mut c_char,
    extname: *const c_char,
    status: *mut c_int,
) -> c_int {
    ffcrtb(
        fptr, tbltype, naxis2, tfields, ttype, tform, tunit, extname, status,
    )
}

pub(crate) unsafe fn fits_delete_col(
    fptr: *mut fitsfile,
    numcol: c_int,
    status: *mut c_int,
) -> c_int {
    ffdcol(fptr, numcol, status)
}

pub(crate) unsafe fn fits_delete_hdu(
    fptr: *mut fitsfile,
    hdutype: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffdhdu(fptr, hdutype, status)
}

pub(crate) unsafe fn fits_file_mode(
    fptr: *mut fitsfile,
    filemode: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffflmd(fptr, filemode, status)
}

pub(crate) unsafe fn fits_get_bcolparms(
    fptr: *mut fitsfile,
    colnum: c_int,
    ttype: *mut c_char,
    tunit: *mut c_char,
    dtype: *mut c_char,
    repeat: *mut c_long,
    tscal: *mut c_double,
    tzero: *mut c_double,
    tnull: *mut c_long,
    tdisp: *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffgbcl(
        fptr, colnum, ttype, tunit, dtype, repeat, tscal, tzero, tnull, tdisp, status,
    )
}

pub(crate) unsafe fn fits_get_col_display_width(
    fptr: *mut fitsfile,
    colnum: c_int,
    width: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcdw(fptr, colnum, width, status)
}

pub(crate) unsafe fn fits_get_colnum(
    fptr: *mut fitsfile,
    casesen: c_int,
    templt: *mut c_char,
    colnum: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcno(fptr, casesen, templt, colnum, status)
}

pub(crate) unsafe fn fits_read_col_str(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: *mut c_char,
    array: *mut *mut c_char,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvs(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_sht(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_short,
    array: *mut c_short,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvi(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_usht(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_ushort,
    array: *mut c_ushort,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvui(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_int(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_int,
    array: *mut c_int,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvk(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_uint(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_uint,
    array: *mut c_uint,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvuk(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_flt(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_float,
    array: *mut c_float,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcve(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_dbl(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_double,
    array: *mut c_double,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvd(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_lng(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_long,
    array: *mut c_long,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvj(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

// int CFITS_API ffgcvjj(fitsfile *fptr, int colnum, LONGLONG firstrow, LONGLONG firstelem,
//            LONGLONG nelem, LONGLONG nulval, LONGLONG *array, int *anynul,
//                       int *status);
pub(crate) unsafe fn fits_read_col_lnglng(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: LONGLONG,
    array: *mut LONGLONG,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvjj(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_ulng(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_ulong,
    array: *mut c_ulong,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvuj(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_col_ulnglng(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: c_ulonglong,
    array: *mut c_ulonglong,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgcvujj(
        fptr, colnum, firstrow, firstelem, nelem, nulval, array, anynul, status,
    )
}
pub(crate) unsafe fn fits_read_key_log(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: *mut c_int,
    comm: *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffgkyl(fptr, keyname, value, comm, status)
}

pub(crate) unsafe fn fits_read_key_lng(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: *mut c_long,
    comm: *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffgkyj(fptr, keyname, value, comm, status)
}

// int CFITS_API ffgkyjj(fitsfile *fptr, const char *keyname, LONGLONG *value, char *comm, int *status);
pub(crate) unsafe fn fits_read_key_lnglng(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: *mut LONGLONG,
    comm: *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffgkyjj(fptr, keyname, value, comm, status)
}

pub(crate) unsafe fn fits_read_key_flt(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: *mut c_float,
    comm: *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffgkye(fptr, keyname, value, comm, status)
}

pub(crate) unsafe fn fits_read_key_dbl(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: *mut c_double,
    comm: *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffgkyd(fptr, keyname, value, comm, status)
}

pub(crate) unsafe fn fits_get_hdu_num(fptr: *mut fitsfile, chdunum: *mut c_int) -> c_int {
    ffghdn(fptr, chdunum)
}

pub(crate) unsafe fn fits_get_hdu_type(
    fptr: *mut fitsfile,
    exttype: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffghdt(fptr, exttype, status)
}

pub(crate) unsafe fn fits_get_img_dim(
    fptr: *mut fitsfile,
    naxis: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgidm(fptr, naxis, status)
}

pub(crate) unsafe fn fits_get_img_equivtype(
    fptr: *mut fitsfile,
    imgtype: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgiet(fptr, imgtype, status)
}

pub(crate) unsafe fn fits_get_img_size(
    fptr: *mut fitsfile,
    nlen: c_int,
    naxes: *mut c_long,
    status: *mut c_int,
) -> c_int {
    ffgisz(fptr, nlen, naxes, status)
}

pub(crate) unsafe fn fits_read_key_str(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: *mut c_char,
    comm: *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffgkys(fptr, keyname, value, comm, status)
}

pub(crate) unsafe fn fits_get_num_cols(
    fptr: *mut fitsfile,
    ncols: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgncl(fptr, ncols, status)
}

pub(crate) unsafe fn fits_get_num_rows(
    fptr: *mut fitsfile,
    nrows: *mut c_long,
    status: *mut c_int,
) -> c_int {
    ffgnrw(fptr, nrows, status)
}

pub(crate) unsafe fn fits_read_img(
    fptr: *mut fitsfile,
    datatype: c_int,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    nulval: *mut c_void,
    array: *mut c_void,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgpv(
        fptr, datatype, firstelem, nelem, nulval, array, anynul, status,
    )
}

pub(crate) unsafe fn fits_read_subset(
    fptr: *mut fitsfile,
    datatype: c_int,
    blc: *mut c_long,
    trc: *mut c_long,
    inc: *mut c_long,
    nulval: *mut c_void,
    array: *mut c_void,
    anynul: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffgsv(fptr, datatype, blc, trc, inc, nulval, array, anynul, status)
}

pub(crate) unsafe fn fits_insert_col(
    fptr: *mut fitsfile,
    numcol: c_int,
    ttype: *mut c_char,
    tform: *mut c_char,
    status: *mut c_int,
) -> c_int {
    fficol(fptr, numcol, ttype, tform, status)
}

pub(crate) unsafe fn fits_movabs_hdu(
    fptr: *mut fitsfile,
    hdunum: c_int,
    exttype: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffmahd(fptr, hdunum, exttype, status)
}

pub(crate) unsafe fn fits_movnam_hdu(
    fptr: *mut fitsfile,
    exttype: c_int,
    hduname: *mut c_char,
    hduvers: c_int,
    status: *mut c_int,
) -> c_int {
    ffmnhd(fptr, exttype, hduname, hduvers, status)
}

pub(crate) unsafe fn fits_create_file(
    fptr: *mut *mut fitsfile,
    filename: *const c_char,
    status: *mut c_int,
) -> c_int {
    ffinit(fptr, filename, status)
}

pub(crate) unsafe fn fits_write_col(
    fptr: *mut fitsfile,
    datatype: c_int,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    array: *mut c_void,
    status: *mut c_int,
) -> c_int {
    ffpcl(
        fptr, datatype, colnum, firstrow, firstelem, nelem, array, status,
    )
}

pub(crate) unsafe fn fits_write_col_str(
    fptr: *mut fitsfile,
    colnum: c_int,
    firstrow: LONGLONG,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    array: *mut *mut c_char,
    status: *mut c_int,
) -> c_int {
    ffpcls(fptr, colnum, firstrow, firstelem, nelem, array, status)
}

pub(crate) unsafe fn fits_write_imghdr(
    fptr: *mut fitsfile,
    bitpix: c_int,
    naxis: c_int,
    naxes: *mut c_long,
    status: *mut c_int,
) -> c_int {
    ffphps(fptr, bitpix, naxis, naxes, status)
}

pub(crate) unsafe fn fits_write_key_flt(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: c_float,
    decim: c_int,
    comm: *const c_char,
    status: *mut c_int,
) -> c_int {
    ffpkye(fptr, keyname, value, decim, comm, status)
}

pub(crate) unsafe fn fits_write_key_dbl(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: c_double,
    decim: c_int,
    comm: *const c_char,
    status: *mut c_int,
) -> c_int {
    ffpkyd(fptr, keyname, value, decim, comm, status)
}
pub(crate) unsafe fn fits_write_key_str(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: *const c_char,
    comm: *const c_char,
    status: *mut c_int,
) -> c_int {
    ffpkys(fptr, keyname, value, comm, status)
}

pub(crate) unsafe fn fits_write_key_log(
    fptr: *mut fitsfile,
    keyname: *const c_char,
    value: c_int,
    comm: *const c_char,
    status: *mut c_int,
) -> c_int {
    ffpkyl(fptr, keyname, value, comm, status)
}

pub(crate) unsafe fn fits_write_img(
    fptr: *mut fitsfile,
    datatype: c_int,
    firstelem: LONGLONG,
    nelem: LONGLONG,
    array: *mut c_void,
    status: *mut c_int,
) -> c_int {
    ffppr(fptr, datatype, firstelem, nelem, array, status)
}

pub(crate) unsafe fn fits_write_subset(
    fptr: *mut fitsfile,
    datatype: c_int,
    fpixel: *mut c_long,
    lpixel: *mut c_long,
    array: *mut c_void,
    status: *mut c_int,
) -> c_int {
    ffpss(fptr, datatype, fpixel, lpixel, array, status)
}

pub(crate) unsafe fn fits_resize_img(
    fptr: *mut fitsfile,
    bitpix: c_int,
    naxis: c_int,
    naxes: *mut c_long,
    status: *mut c_int,
) -> c_int {
    ffrsim(fptr, bitpix, naxis, naxes, status)
}

pub(crate) unsafe fn fits_get_num_hdus(
    fptr: *mut fitsfile,
    nhdu: *mut c_int,
    status: *mut c_int,
) -> c_int {
    ffthdu(fptr, nhdu, status)
}

pub(crate) unsafe fn fits_open_file(
    fptr: *mut *mut fitsfile,
    filename: *const c_char,
    iomode: c_int,
    status: *mut c_int,
) -> c_int {
    ffopen(fptr, filename, iomode, status)
}

pub(crate) unsafe fn fits_write_key(
    fptr: *mut fitsfile,
    datatype: c_int,
    keyname: *const c_char,
    value: *mut c_void,
    comm: *const c_char,
    status: *mut c_int,
) -> c_int {
    ffpky(fptr, datatype, keyname, value, comm, status)
}
