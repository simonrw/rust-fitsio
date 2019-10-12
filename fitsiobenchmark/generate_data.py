#!/usr/bin/env python


import fitsio
import numpy as np
from functools import partial


BIAS_LEVEL = 2000
RAW_IMAGE_SHAPE = (2048, 2048)


def generate_frame(
    filename, shape, intdata=True, overscan=False, add_bias=False, normalise=False
):
    if overscan:
        shape = (shape[0] + 40, shape[1])

    if intdata:
        if add_bias:
            bias = np.random.randint(0, BIAS_LEVEL, size=shape)
            data = np.random.randint(BIAS_LEVEL, 2 ** 16 - 1, size=shape) + bias
        else:
            data = np.random.randint(0, 2 ** 16 - 1, size=shape)
    else:
        if add_bias:
            bias = np.random.randint(0, BIAS_LEVEL, size=shape)
            data = np.random.uniform(BIAS_LEVEL, 2 ** 16 - 1, size=shape) + bias
        else:
            data = np.random.uniform(0, 2 ** 16 - 1, size=shape)

    if normalise:
        r = data.ptp()
        data = (data - data.min()) / r

        # Put within normal flat range
        data = (data * 0.3) + 0.7

    with fitsio.FITS(filename, fitsio.READWRITE, clobber=True) as outfile:
        outfile.write(data)


generate_bias = partial(
    generate_frame, shape=RAW_IMAGE_SHAPE, intdata=True,
)
generate_dark = partial(
    generate_frame, shape=RAW_IMAGE_SHAPE, intdata=False, overscan=False
)
generate_flat = partial(
    generate_frame, shape=RAW_IMAGE_SHAPE, intdata=False, overscan=False, normalise=True
)
generate_image = partial(
    generate_frame, shape=RAW_IMAGE_SHAPE, intdata=False, overscan=False, add_bias=True
)


if __name__ == "__main__":
    generate_bias("bias.fits")
    generate_dark("dark.fits")
    generate_flat("flat.fits")
    generate_image("science.fits")
