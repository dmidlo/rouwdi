#ifndef ROUWDI_WASI_SDK_SHIM_STDLIB_H
#define ROUWDI_WASI_SDK_SHIM_STDLIB_H

#include_next <stdlib.h>

#if defined(__wasi__)
static inline unsigned int arc4random(void) {
  return 0;
}
#endif

#endif
