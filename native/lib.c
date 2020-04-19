#include <stdio.h>
#include <string.h>
#include <gc.h>
#include <unistd.h>
#include <sys/wait.h>

struct Process {
  int pid;
  int in;
  int out;
  int err;
};

struct Process* lilit_exec(char *cmd) {
   int* in = GC_malloc(sizeof(int) * 2);
   int* out = GC_malloc(sizeof(int) * 2);
   int* err = GC_malloc(sizeof(int) * 2);

   pipe(in);
   pipe(out);
   pipe(err);


   int pid = fork();

  if (pid == 0) {
    close(in[1]);
    close(out[0]);
    close(err[0]);
    dup2(in[0], STDIN_FILENO);
    dup2(out[1], STDOUT_FILENO);
    dup2(err[1], STDERR_FILENO);

    execlp(cmd, cmd, (char*) NULL);
  }
  close(in[0]);
  close(out[1]);
  close(err[1]);

  struct Process* process = GC_malloc(sizeof(struct Process));
  process->pid = pid;
  process->in = in[1];
  process->out = 10;
  process->err = err[0];
  return process;
}

char lilit_read(int pipe) {
  char c;
  read(pipe, &c, 1);
  return c;
}

void lilit_write(int pipe, char c) {
  write(pipe, &c, 1);
}

int lilit_wait(int pid) {
  int exitCode;
  waitpid(pid, &exitCode, 0);
  return exitCode;
}


struct Process* test_call() {
  struct Process* process = GC_malloc(sizeof(struct Process));
  process->pid = 2;
  process->in = 3;
  process->out = 10;
  process->err = 9;
  return process;
}

int GC_finalizer_count = 0;

void GC_finalizer(char* a, char* b) {
  GC_finalizer_count++;
  printf("GC freed count: %d\n", GC_finalizer_count);
}