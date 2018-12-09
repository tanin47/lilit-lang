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
   printf(t.content);
   return 0;
}
