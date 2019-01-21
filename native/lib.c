#include <stdio.h>
#include <string.h>
#include <gc.h>

char* lilit__read() {
  int block_size = 3;
  int size = block_size;
  char* buf = GC_malloc(sizeof(char) * size);

  char* offset = buf;

  while (fgets(offset, block_size, stdin)) {
    int len = strlen(offset);

    if (len == (block_size - 1) && offset[len - 1] != '\n') {
      int new_size = size + block_size - 1;
      buf = GC_realloc(buf, sizeof(char) * new_size);
      offset = &buf[size - 1];
      size = new_size;
    } else {
      offset[len - 1] = '\0';
      return buf;
    }
  }

  return buf;
}

char* lilit__read_file(char* filename) {
  FILE *file = fopen(filename, "r");

  int block_size = 11;
  int size = block_size;
  char* buf = GC_malloc(sizeof(char) * size);

  int nread = 0;
  char* offset = buf;

  while ((nread = fread(offset, sizeof(char), block_size, file)) > 0) {
    int new_size = size + block_size;
    buf = GC_realloc(buf, sizeof(char) * new_size);
    offset = &buf[size];
    size = new_size;
  }

  return buf;
}

int GC_finalizer_count = 0;

void GC_finalizer(char* a, char* b) {
  GC_finalizer_count++;
  printf("GC freed count: %d\n", GC_finalizer_count);
}