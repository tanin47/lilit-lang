struct I32 {
  int n;
};

struct Number {
  struct I32* i32;
};

struct I32* test(struct Number *a, struct Number *b) {
  return a->i32;
}

int main() {
  struct Number number;
  struct I32 i32;
  i32.n = 123;
  number.i32 = &i32;

  struct Number number_b;


  return test(&number, &number_b)->n;
};
