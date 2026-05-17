#ifndef ROUWDI_WASI_SDK_SHIM_BITS_SIGNAL_H
#define ROUWDI_WASI_SDK_SHIM_BITS_SIGNAL_H

#include_next <bits/signal.h>

#ifndef SA_NODEFER
#define SA_NODEFER 0x40000000
#endif
#ifndef SA_RESETHAND
#define SA_RESETHAND 0x80000000
#endif
#ifndef SA_ONSTACK
#define SA_ONSTACK 0x08000000
#endif
#ifndef SA_SIGINFO
#define SA_SIGINFO 4
#endif

#endif
