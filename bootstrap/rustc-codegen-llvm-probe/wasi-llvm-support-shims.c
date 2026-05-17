#include <dlfcn.h>
#include <errno.h>
#include <signal.h>
#include <stddef.h>
#include <sys/resource.h>
#include <sys/types.h>
#include <unistd.h>

static const char ROUWDI_DLOPEN_UNAVAILABLE[] =
    "dynamic loading is unavailable inside the rouwdi wasm LLVM payload";

__attribute__((weak)) int dlclose(void *handle) {
  (void)handle;
  return 0;
}

__attribute__((weak)) char *dlerror(void) {
  return (char *)ROUWDI_DLOPEN_UNAVAILABLE;
}

__attribute__((weak)) void *dlopen(const char *filename, int flags) {
  (void)filename;
  (void)flags;
  errno = ENOSYS;
  return NULL;
}

__attribute__((weak)) void *dlsym(void *handle, const char *symbol) {
  (void)handle;
  (void)symbol;
  errno = ENOSYS;
  return NULL;
}

__attribute__((weak)) int dup2(int oldfd, int newfd) {
  (void)oldfd;
  (void)newfd;
  errno = ENOSYS;
  return -1;
}

__attribute__((weak)) int execv(const char *path, char *const argv[]) {
  (void)path;
  (void)argv;
  errno = ENOSYS;
  return -1;
}

__attribute__((weak)) int execve(const char *path, char *const argv[],
                                 char *const envp[]) {
  (void)path;
  (void)argv;
  (void)envp;
  errno = ENOSYS;
  return -1;
}

__attribute__((weak)) int execvp(const char *file, char *const argv[]) {
  (void)file;
  (void)argv;
  errno = ENOSYS;
  return -1;
}

__attribute__((weak)) pid_t fork(void) {
  errno = ENOSYS;
  return -1;
}

__attribute__((weak)) pid_t getpid(void) {
  return 1;
}

__attribute__((weak)) uid_t getuid(void) {
  return 0;
}

__attribute__((weak)) int getrlimit(int resource, struct rlimit *rlim) {
  (void)resource;
  if (rlim) {
    rlim->rlim_cur = RLIM_INFINITY;
    rlim->rlim_max = RLIM_INFINITY;
  }
  return 0;
}

__attribute__((weak)) int posix_madvise(void *addr, size_t len, int advice) {
  (void)addr;
  (void)len;
  (void)advice;
  return 0;
}

__attribute__((weak)) unsigned int alarm(unsigned int seconds) {
  (void)seconds;
  return 0;
}

__attribute__((weak)) int setsid(void) {
  errno = ENOSYS;
  return -1;
}

__attribute__((weak)) int setrlimit(int resource, const struct rlimit *rlim) {
  (void)resource;
  (void)rlim;
  return 0;
}

__attribute__((weak)) int sigaction(int signum, const struct sigaction *act,
                                    struct sigaction *oldact) {
  (void)act;
  if (signum < 0) {
    errno = EINVAL;
    return -1;
  }
  if (oldact) {
    oldact->sa_handler = SIG_DFL;
    oldact->sa_mask = 0;
    oldact->sa_flags = 0;
  }
  return 0;
}

__attribute__((weak)) int sigemptyset(sigset_t *set) {
  if (set) {
    *set = 0;
  }
  return 0;
}

__attribute__((weak)) int sigfillset(sigset_t *set) {
  if (set) {
    *set = ~(sigset_t)0;
  }
  return 0;
}

__attribute__((weak)) int sigprocmask(int how, const sigset_t *set,
                                      sigset_t *oldset) {
  (void)how;
  (void)set;
  if (oldset) {
    *oldset = 0;
  }
  return 0;
}
