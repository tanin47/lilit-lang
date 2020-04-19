#define _GNU_SOURCE
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/wait.h>

extern char **environ;

int main() {
  int size = 0;
  while (environ[size]) size++;
  printf("Size: %d\n", size);

  char **env = malloc((size + 1) * sizeof(char *));

  for (int i=0;i<size;i++) {
    int strSize = strlen(environ[i]);
    env[i] = malloc((strSize + 1) * sizeof(char));
    strcpy(env[i], environ[i]);
  }

  env[size] = (char *)0;

   int in[2];
   pipe(in);
   int out[2];
   pipe(out);
   int err[2];
   pipe(err);

  if (fork() == 0) {
    printf("Child Start.\n");
    close(in[1]);
    close(out[0]);
    close(err[0]);
    dup2(in[0], STDIN_FILENO);
    dup2(out[1], STDOUT_FILENO);
    dup2(err[1], STDERR_FILENO);

    char *cmd[] = { "./bash.sh", (char *)0 };
    printf("Child exec.\n");
    execvpe("./bash.sh", cmd, env);
    // The process is replaced.
    printf("This doesn't print\n");
  } else {
    printf("Parent starts\n");

    close(in[0]);
    close(out[1]);
    close(err[1]);

    int outPid = fork();

    if (outPid == 0) {
      printf("Out starts\n");
      int bufsize = 10;
      int readSize;
      char inbuf[bufsize];
      int printMarker = 1;
      while ((readSize = read(out[0], inbuf, bufsize)) != 0) {
        if (printMarker) {
          printf("out> ");
          printMarker = 0;
        }
        for (int i=0;i<readSize;i++) {
          if (inbuf[i] == '\n') { printMarker = 1; }
          printf("%c", inbuf[i]);
        }
      }
      return 0;
    }

    int errPid = fork();

    if (errPid == 0) {
      printf("Err starts\n");
      int bufsize = 10;
      int readSize;
      char inbuf[bufsize];
      int printMarker = 1;
      while ((readSize = read(err[0], inbuf, bufsize)) != 0) {
        if (printMarker) {
          printf("err> ");
          printMarker = 0;
        }
        for (int i=0;i<readSize;i++) {
          if (inbuf[i] == '\n') { printMarker = 1; }
          printf("%c", inbuf[i]);
        }
      }

      return 0;
    }

    printf("In starts\n");
    char message[] = "Test\n";
    write(in[1], message, strlen(message) + 1);
    close(in[0]);

    int returnStatus;
    waitpid(outPid, &returnStatus, 0);
    waitpid(errPid, &returnStatus, 0);

    printf("Parent exits\n");
  }
  printf("Exit.");
}
