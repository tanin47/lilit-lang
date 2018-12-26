#include <stdio.h>

struct Foo { int a; };

int main() {
  struct Foo foo;
  printf("%p\n", (void *)&foo);
  return 0;
}