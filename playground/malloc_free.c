#include <stdlib.h>

struct Foo {
  int a;
};

int main() {
  struct Foo* foo = malloc(sizeof(struct Foo));
  foo->a = 123;
  free(foo);
  return 0;
}