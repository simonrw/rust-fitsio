#include <longnam.h>
#include <stdio.h>
#include <fitsio.h>
#include <assert.h>

int main() {
    int status = 0;
    fitsfile* fptr = NULL;

    if (fits_create_file(&fptr, "!test.fits", &status)) {
        fits_report_error(stderr, status);
        return status;
    }

    // Create the primary HDU
    long naxes[] = {100, 100};
    if (fits_create_img(fptr, USHORT_IMG, 2, naxes, &status)) {
        fits_report_error(stderr, status);
        return status;
    }

    // Create the table
    const int tfields = 2;
    char* ttype[] = {"A", "B"};
    char* tform[] = {"10I", "I"};
    char* tunit[] = {"", ""};
    if (fits_create_tbl(fptr, BINARY_TBL, 0, tfields, ttype, tform, tunit, NULL, &status)) {
        fits_report_error(stderr, status);
        return status;
    }
    if (fits_write_key(fptr, TSTRING, "EXTNAME", "FOO", NULL, &status)) {
        fits_report_error(stderr, status);
        return status;
    }

    // Add some data
    int data[] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19};
    const int nelements = sizeof(data) / sizeof(data[0]);
    if (fits_write_col(fptr, TINT, 1, 1, 1, nelements, data, &status)) {
        fits_report_error(stderr, status);
        return status;
    }



    if (fits_close_file(fptr, &status)) {
        fits_report_error(stderr, status);
        return status;
    }

    assert(status == 0);

    return 0;
}
