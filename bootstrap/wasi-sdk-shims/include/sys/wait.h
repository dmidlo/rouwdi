#ifndef ROUWDI_WASI_SDK_SHIM_SYS_WAIT_H
#define ROUWDI_WASI_SDK_SHIM_SYS_WAIT_H

#include <errno.h>
#include <sys/types.h>

#define WIFEXITED(status) ((status) >= 0 && (status) < 256)
#define WEXITSTATUS(status) (status)
#define WIFSIGNALED(status) 0
#define WTERMSIG(status) 0
#define WNOHANG 1

static inline pid_t wait(int *status) {
  if (status) {
    *status = 0;
  }
  errno = ENOSYS;
  return -1;
}

struct rusage;

static inline pid_t wait4(pid_t pid, int *status, int options,
                          struct rusage *usage) {
  (void)pid;
  (void)options;
  (void)usage;
  if (status) {
    *status = 0;
  }
  errno = ENOSYS;
  return -1;
}

#endif
