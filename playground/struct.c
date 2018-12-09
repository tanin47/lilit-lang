#include <stdio.h>

struct Test {
  int size;
  char* content;
};

int main()
{
   char content[10];
   struct Test t;
   t.size = 11;
   t.content = content;
   struct Test* a = &t;
   printf(a->content);
   return 0;
}
