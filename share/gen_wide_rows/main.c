#include <assert.h>
#include <fitsio.h>
#include <longnam.h>
#include <stdio.h>
#include <string.h>

int write_file(char *filename);
int read_file(char *filename);

int main() {
  char *filename = "test.fits";
  int ret = 0;
  ret = write_file(filename);
  if (ret != 0) {
    return ret;
  }
  ret = read_file(filename);
  if (ret != 0) {
    return ret;
  }
  fprintf(stderr, "done\n");
  return 0;
}

int write_file(char *filename) {
  int status = 0;
  fitsfile *fptr = NULL;

  fprintf(stderr, "writing file\n");

  // we have to prepend an exclamation mark to ensure we overwrite any existing
  // file
  size_t filename_length = strlen(filename);
  char buf[4096];
  snprintf(buf, filename_length + 2, "!%s", filename);

  if (fits_create_file(&fptr, buf, &status)) {
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
  char *ttype[] = {"A", "B"};
  char *tform[] = {"10I", "I"};
  char *tunit[] = {"", ""};
  if (fits_create_tbl(fptr, BINARY_TBL, 0, tfields, ttype, tform, tunit, NULL,
                      &status)) {
    fits_report_error(stderr, status);
    return status;
  }
  if (fits_write_key(fptr, TSTRING, "EXTNAME", "FOO", NULL, &status)) {
    fits_report_error(stderr, status);
    return status;
  }

  // Add some data
  int data[] = {0,  1,  2,  3,  4,  5,  6,  7,  8,  9,
                10, 11, 12, 13, 14, 15, 16, 17, 18, 19};
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

int read_file(char *filename) {
  int status = 0;
  fitsfile *fptr = NULL;

  fprintf(stderr, "opening file\n");
  if (fits_open_file(&fptr, filename, READONLY, &status)) {
    fits_report_error(stderr, status);
    return status;
  }

  if (fits_movnam_hdu(fptr, BINARY_TBL, "FOO", 0, &status)) {
    fits_report_error(stderr, status);
    return status;
  }

  long num_rows = 0;
  if (fits_get_num_rows(fptr, &num_rows, &status)) {
    fits_report_error(stderr, status);
    return status;
  }

  fprintf(stderr, "found %ld rows\n", num_rows);

  int colnum = 0;
  if (fits_get_colnum(fptr, CASEINSEN, "a", &colnum, &status)) {
    fits_report_error(stderr, status);
    return status;
  }

  fprintf(stderr, "found column %d\n", colnum);

  int typecode = 0;
  long repeat = 0;
  long width = 0;
  if (fits_get_coltype(fptr, colnum, &typecode, &repeat, &width, &status)) {
    fits_report_error(stderr, status);
    return status;
  }

  fprintf(stderr, "column has type %d, with repeat %ld and width %ld\n",
          typecode, repeat, width);

  switch (typecode) {
  case TSHORT: {
    fprintf(stderr, "TSHORT column\n");
    short *buf = malloc(num_rows * repeat * sizeof(short));
    if (fits_read_col(fptr, typecode, colnum, 1, 1, num_rows * repeat, NULL,
                      buf, NULL, &status)) {
      fits_report_error(stderr, status);
      return status;
    }

    for (int i = 0; i < num_rows * repeat; i++) {
      if (i == 0) {
        fprintf(stderr, "%d", buf[i]);
      } else {
        fprintf(stderr, ", %d", buf[i]);
      }
    }
    fprintf(stderr, "\n");

    if (buf)
      free(buf);
    break;
  }
  default:
    fprintf(stderr, "unknown item type %d\n", typecode);
    return 1;
    break;
  }

  if (fits_close_file(fptr, &status)) {
    fits_report_error(stderr, status);
    return status;
  }

  return 0;
}
