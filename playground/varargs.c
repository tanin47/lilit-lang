#include <stdio.h>
#include <stdarg.h>

void print(char *s, ...) {
  va_list args;
  va_start(args, s);
  vprintf("%d %d\n", args);
  va_end(args);
}