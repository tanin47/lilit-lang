int main(int argc, char** args) {
  char* new_args[argc];

  for (int i = 0;i < argc;i++) {
    new_args[i] = args[i];
  }

  return 1;
}