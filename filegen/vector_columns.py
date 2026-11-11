#!/usr/bin/env python

"""
Generate a file with vector columns, to validate https://github.com/simonrw/rust-fitsio/pull/330
"""

import argparse
from pathlib import Path

import numpy as np
from astropy.io import fits


def gen_file() -> fits.BinTableHDU:
    # https://docs.astropy.org/en/stable/io/fits/index.html#creating-a-new-table-file
    a1 = np.array(["NGC1001", "NGC1002", "NGC1003"])
    a2 = np.array([11.1, 12.3, 15.2])
    a3 = np.array([[1, 2], [3, 4], [5, 6]])
    col1 = fits.Column(name="target", format="20A", array=a1)
    col2 = fits.Column(name="V_mag", format="E", array=a2)
    col3 = fits.Column(name="index", format="2K", array=a3)
    cols = fits.ColDefs([col1, col2, col3])
    hdu = fits.BinTableHDU.from_columns(cols, name="info")
    return hdu


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-o", "--output", required=True, type=Path)
    args = parser.parse_args()

    file = gen_file()
    file.writeto(str(args.output), overwrite=True)
