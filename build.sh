cargo run examples/native.l \
  && llc-6.0 -filetype=obj native/lib.ll \
  && cc native/lib.o output/main.o ~/projects/bdwgc/.libs/libgc.so -I ~/projects/bdwgc/include/ -o main -no-pie \
  && ./main