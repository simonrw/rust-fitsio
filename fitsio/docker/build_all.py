#!/usr/bin/env python3.6


import subprocess as sp


VERSIONS = [
        '3080',
        '3080',
        '3090',
        '3130',
        '3140',
        '3181',
        '3200',
        '3210',
        '3230',
        '3240',
        '3250',
        '3260',
        '3270',
        '3280',
        '3290',
        '3300',
        '3310',
        '3330',
        '3340',
        '3350',
        '3360',
        '3370',
        '3380',
        '3390',
        '3410',
        '3420',
        ]

TEMPLATE_URL = 'ftp://heasarc.gsfc.nasa.gov/software/fitsio/c/cfitsio{version}.tar.gz'


with open('Dockerfile.template') as infile:
    template = infile.read()


for version in VERSIONS:
    url = TEMPLATE_URL.format(version=version)
    text = template.format(url=url, version=version)

    out_filename = 'Dockerfile.{version}'.format(version=version)
    with open(out_filename, 'w') as outfile:
        outfile.write(text)

    cmd = ['make', f'VERSION={version}', f'DOCKERFILE={out_filename}']
    sp.check_call(cmd)
