#!/usr/bin/env python


import time
import fitsio


def read_from_fits(filename):
    with fitsio.FITS(filename) as infile:
        return infile[0].read()


def timeit(f, n=64):
    times = []
    for _ in range(n):
        start_time = time.time()
        f()
        end_time = time.time()
        times.append(end_time - start_time)
    return times


def main():
    bias = read_from_fits("bias.fits")
    dark = read_from_fits("dark.fits")
    flat = read_from_fits("flat.fits")
    science = read_from_fits("science.fits")

    result = (science - bias - dark) / flat

if __name__ == "__main__":
    times = timeit(main, n=64)
    for time in times:
        print(time)
