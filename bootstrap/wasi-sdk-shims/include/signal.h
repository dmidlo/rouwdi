#ifndef ROUWDI_WASI_SDK_SHIM_SIGNAL_H
#define ROUWDI_WASI_SDK_SHIM_SIGNAL_H

#include_next <signal.h>

#if defined(__wasi__) && !defined(__wasilibc_unmodified_upstream)

#include <errno.h>

#ifndef SIG_BLOCK
#define SIG_BLOCK 0
#endif
#ifndef SIG_UNBLOCK
#define SIG_UNBLOCK 1
#endif
#ifndef SIG_SETMASK
#define SIG_SETMASK 2
#endif

struct sigaction {
  void (*sa_handler)(int);
  sigset_t sa_mask;
  int sa_flags;
};

static inline int sigemptyset(sigset_t *set) {
  if (set) {
    *set = 0;
  }
  return 0;
}

static inline int sigaddset(sigset_t *set, int signum) {
  if (!set || signum < 0) {
    errno = EINVAL;
    return -1;
  }
  *set = (sigset_t)(*set | (sigset_t)(1u << (signum & ((int)(sizeof(sigset_t) * 8) - 1))));
  return 0;
}

static inline int sigprocmask(int how, const sigset_t *set, sigset_t *oldset) {
  (void)how;
  (void)set;
  if (oldset) {
    *oldset = 0;
  }
  return 0;
}

static inline int sigaction(int signum, const struct sigaction *act,
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

#endif

#endif
