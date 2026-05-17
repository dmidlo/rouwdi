#ifndef ROUWDI_WASI_SDK_SHIM_UNISTD_H
#define ROUWDI_WASI_SDK_SHIM_UNISTD_H

#include_next <unistd.h>

#if defined(__wasi__) && !defined(__wasilibc_unmodified_upstream)

#include <errno.h>

static inline int dup(int fd) {
  (void)fd;
  errno = ENOSYS;
  return -1;
}

static inline int dup2(int oldfd, int newfd) {
  (void)oldfd;
  (void)newfd;
  errno = ENOSYS;
  return -1;
}

static inline pid_t getsid(pid_t pid) {
  (void)pid;
  errno = ENOSYS;
  return -1;
}

#endif

#endif
