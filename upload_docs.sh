#!/bin/bash
set -e

export CARGO_TARGET_DIR=target

ghp-import -n -p target/doc
