#ifndef ROUWDI_WASI_SDK_SHIM_BITS_FCNTL_H
#define ROUWDI_WASI_SDK_SHIM_BITS_FCNTL_H

#ifndef O_APPEND
#define O_APPEND 0x0008
#endif
#ifndef O_CREAT
#define O_CREAT 0x0100
#endif
#ifndef O_EXCL
#define O_EXCL 0x0400
#endif
#ifndef O_TRUNC
#define O_TRUNC 0x0200
#endif
#ifndef O_NOFOLLOW
#define O_NOFOLLOW 0x1000000
#endif
#ifndef O_DIRECTORY
#define O_DIRECTORY 0x2000000
#endif
#ifndef O_NONBLOCK
#define O_NONBLOCK 0x4000
#endif
#ifndef O_NDELAY
#define O_NDELAY O_NONBLOCK
#endif
#ifndef O_CLOEXEC
#define O_CLOEXEC 0x200000
#endif
#ifndef O_SYNC
#define O_SYNC 0x101000
#endif
#ifndef O_DSYNC
#define O_DSYNC 0x1000
#endif
#ifndef O_RSYNC
#define O_RSYNC O_SYNC
#endif
#ifndef O_PATH
#define O_PATH 0x200000
#endif

#ifndef F_DUPFD
#define F_DUPFD 0
#endif
#ifndef F_GETFD
#define F_GETFD 1
#endif
#ifndef F_SETFD
#define F_SETFD 2
#endif
#ifndef F_GETFL
#define F_GETFL 3
#endif
#ifndef F_SETFL
#define F_SETFL 4
#endif
#ifndef F_GETLK
#define F_GETLK 5
#endif
#ifndef F_SETLK
#define F_SETLK 6
#endif
#ifndef F_SETLKW
#define F_SETLKW 7
#endif

#endif
