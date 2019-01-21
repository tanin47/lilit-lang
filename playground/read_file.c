#include <stdio.h>

int main() {
  char buf[1024];
  size_t nread;

  FILE* file = fopen("test.txt", "r");
  nread = fread(buf, 1, sizeof buf, file);

  return 0;
}