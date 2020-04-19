#define _GNU_SOURCE
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/wait.h>

extern char **environ;

int main() {
  execlp("ls", "ls", (char*)NULL);
}
