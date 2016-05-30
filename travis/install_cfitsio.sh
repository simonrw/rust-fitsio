#!/usr/bin/env bash
# -*- coding: utf-8 -*-

set -ex

CFITSIOVERSION=3390
TARBALLNAME=cfitsio${CFITSIOVERSION}.tar.gz
DIRNAME=cfitsio-${CFITSIOVERSION}

wget -c http://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/${TARBALLNAME}
tar -xzvf ${TARBALLNAME}
mv cfitsio ${DIRNAME}
cd ${DIRNAME} && ./configure --prefix=/usr && make && sudo make install
