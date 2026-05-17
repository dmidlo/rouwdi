#ifndef ROUWDI_WASI_SDK_SHIM_PWD_H
#define ROUWDI_WASI_SDK_SHIM_PWD_H

#include <errno.h>
#include <sys/types.h>

struct passwd {
  char *pw_name;
  char *pw_passwd;
  uid_t pw_uid;
  gid_t pw_gid;
  char *pw_gecos;
  char *pw_dir;
  char *pw_shell;
};

static inline struct passwd *getpwuid(uid_t uid) {
  (void)uid;
  errno = ENOSYS;
  return 0;
}

static inline struct passwd *getpwnam(const char *name) {
  (void)name;
  errno = ENOSYS;
  return 0;
}

static inline int getpwuid_r(uid_t uid, struct passwd *pwd, char *buf,
                             size_t buflen, struct passwd **result) {
  (void)uid;
  (void)pwd;
  (void)buf;
  (void)buflen;
  if (result) {
    *result = 0;
  }
  errno = ENOSYS;
  return ENOSYS;
}

static inline int getpwnam_r(const char *name, struct passwd *pwd, char *buf,
                             size_t buflen, struct passwd **result) {
  (void)name;
  (void)pwd;
  (void)buf;
  (void)buflen;
  if (result) {
    *result = 0;
  }
  errno = ENOSYS;
  return ENOSYS;
}

#endif
