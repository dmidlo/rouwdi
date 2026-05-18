#include "lld/Common/Driver.h"
#include "llvm/ADT/ArrayRef.h"
#include "llvm/Support/raw_ostream.h"

#include <cstdlib>
#include <cstring>
#include <string>

LLD_HAS_DRIVER(wasm)

extern "C" void __wasilibc_initialize_environ(void);
extern "C" void __wasilibc_populate_preopens(void);

extern "C" int rouwdi_lld_wasm_link(int argc, const char *const *argv,
                                    char **stdout_ptr, size_t *stdout_len,
                                    char **stderr_ptr, size_t *stderr_len) {
  std::string stdout_storage;
  std::string stderr_storage;
  llvm::raw_string_ostream stdout_stream(stdout_storage);
  llvm::raw_string_ostream stderr_stream(stderr_storage);
  llvm::ArrayRef<const char *> args(argv, static_cast<size_t>(argc));

  __wasilibc_initialize_environ();
  __wasilibc_populate_preopens();

  bool ok = lld::wasm::link(args, stdout_stream, stderr_stream,
                            /*exitEarly=*/false,
                            /*disableOutput=*/false);
  stdout_stream.flush();
  stderr_stream.flush();

  auto copy_output = [](const std::string &source, char **ptr, size_t *len) {
    if (!ptr || !len) {
      return;
    }
    *len = source.size();
    *ptr = static_cast<char *>(std::malloc(source.size() + 1));
    if (!*ptr) {
      *len = 0;
      return;
    }
    if (!source.empty()) {
      std::memcpy(*ptr, source.data(), source.size());
    }
    (*ptr)[source.size()] = '\0';
  };

  copy_output(stdout_storage, stdout_ptr, stdout_len);
  copy_output(stderr_storage, stderr_ptr, stderr_len);
  return ok ? 0 : 1;
}

extern "C" void rouwdi_lld_free(char *ptr) { std::free(ptr); }
