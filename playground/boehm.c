// clang -S -emit-llvm boehm.c -I ~/projects/bdwgc/include/

#include <stdio.h>
#include "gc.h"

char* read() {
  char buf[10];
  fgets(buf, 10, (void *)NULL);
  return NULL;
}

int main() {
  GC_init();
  return 1;
}