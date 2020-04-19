#!/bin/bash
set -x #echo on

cargo run examples/simple.lilit \
  && clang -S -emit-llvm ~/projects/bdwgc/.libs/libgc.so -I ~/projects/bdwgc/include/ -o native/lib.ll native/lib.c \
  && llc-6.0 -filetype=obj native/lib.ll \
  && cc native/lib.o output/main.o ~/projects/bdwgc/.libs/libgc.so -I ~/projects/bdwgc/include/ -o main -no-pie \
  && ./main
