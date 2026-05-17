@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
for %%I in ("%SCRIPT_DIR%..\..") do set "REPO=%%~fI"
set "LLVM_BUILD=%REPO%\third_party\rust\build\wasm32-wasip1\llvm\build"
set "LLVM_SRC=%REPO%\third_party\rust\src\llvm-project\llvm"
set "SHIMS=%REPO%\bootstrap\wasi-sdk-shims\include"

if "%~1"=="--help" (
  echo rouwdi target llvm-config shim 1>&2
  exit /b 0
)

if "%~1"=="--host-target" (
  echo wasm32-wasip1
  exit /b 0
)

if "%~1"=="--components" (
  echo ipo bitreader bitwriter linker asmparser lto coverage instrumentation webassembly
  exit /b 0
)

if "%~1"=="--cxxflags" (
  echo -I%LLVM_BUILD:\=/%/include -I%LLVM_SRC:\=/%/include -I%SHIMS:\=/% --target=wasm32-wasip1 -std=c++17 -fno-exceptions -fno-rtti -D__STDC_CONSTANT_MACROS -D__STDC_FORMAT_MACROS -D__STDC_LIMIT_MACROS -DNDEBUG -DLLVM_ON_UNIX -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_MMAN -D_WASI_EMULATED_PROCESS_CLOCKS -D__wasilibc_unmodified_upstream -DHAVE_SYS_MMAN_H=1
  exit /b 0
)

if "%~1"=="--includedir" (
  echo %LLVM_BUILD%\include
  exit /b 0
)

if "%~1"=="--libdir" (
  echo %LLVM_BUILD%\lib
  exit /b 0
)

if "%~1"=="--link-static" if "%~2"=="--ldflags" (
  echo -L%LLVM_BUILD:\=/%/lib
  exit /b 0
)

if "%~1"=="--link-static" if "%~2"=="--libs" (
  echo -lLLVMWebAssemblyDisassembler -lLLVMMCDisassembler -lLLVMWebAssemblyAsmParser -lLLVMWebAssemblyCodeGen -lLLVMWebAssemblyUtils -lLLVMWebAssemblyDesc -lLLVMWebAssemblyInfo -lLLVMAsmPrinter -lLLVMCoverage -lLLVMLTO -lLLVMPlugins -lLLVMPasses -lLLVMIRPrinter -lLLVMHipStdPar -lLLVMCoroutines -lLLVMGlobalISel -lLLVMSelectionDAG -lLLVMCFGuard -lLLVMExtensions -lLLVMCodeGen -lLLVMTarget -lLLVMObjCARCOpts -lLLVMCodeGenTypes -lLLVMCGData -lLLVMipo -lLLVMInstrumentation -lLLVMVectorize -lLLVMSandboxIR -lLLVMLinker -lLLVMFrontendOpenMP -lLLVMFrontendDirective -lLLVMFrontendAtomic -lLLVMFrontendOffloading -lLLVMObjectYAML -lLLVMScalarOpts -lLLVMInstCombine -lLLVMBitWriter -lLLVMAggressiveInstCombine -lLLVMTransformUtils -lLLVMAnalysis -lLLVMProfileData -lLLVMSymbolize -lLLVMDebugInfoBTF -lLLVMDebugInfoPDB -lLLVMDebugInfoMSF -lLLVMDebugInfoCodeView -lLLVMDebugInfoGSYM -lLLVMDebugInfoDWARF -lLLVMObject -lLLVMTextAPI -lLLVMMCParser -lLLVMIRReader -lLLVMAsmParser -lLLVMMC -lLLVMDebugInfoDWARFLowLevel -lLLVMBitReader -lLLVMFrontendHLSL -lLLVMCore -lLLVMRemarks -lLLVMBitstreamReader -lLLVMBinaryFormat -lLLVMTargetParser -lLLVMSupport -lLLVMDemangle
  exit /b 0
)

if "%~1"=="--libs" (
  echo -lLLVMWebAssemblyDisassembler -lLLVMMCDisassembler -lLLVMWebAssemblyAsmParser -lLLVMWebAssemblyCodeGen -lLLVMWebAssemblyUtils -lLLVMWebAssemblyDesc -lLLVMWebAssemblyInfo -lLLVMAsmPrinter -lLLVMCoverage -lLLVMLTO -lLLVMPlugins -lLLVMPasses -lLLVMIRPrinter -lLLVMHipStdPar -lLLVMCoroutines -lLLVMGlobalISel -lLLVMSelectionDAG -lLLVMCFGuard -lLLVMExtensions -lLLVMCodeGen -lLLVMTarget -lLLVMObjCARCOpts -lLLVMCodeGenTypes -lLLVMCGData -lLLVMipo -lLLVMInstrumentation -lLLVMVectorize -lLLVMSandboxIR -lLLVMLinker -lLLVMFrontendOpenMP -lLLVMFrontendDirective -lLLVMFrontendAtomic -lLLVMFrontendOffloading -lLLVMObjectYAML -lLLVMScalarOpts -lLLVMInstCombine -lLLVMBitWriter -lLLVMAggressiveInstCombine -lLLVMTransformUtils -lLLVMAnalysis -lLLVMProfileData -lLLVMSymbolize -lLLVMDebugInfoBTF -lLLVMDebugInfoPDB -lLLVMDebugInfoMSF -lLLVMDebugInfoCodeView -lLLVMDebugInfoGSYM -lLLVMDebugInfoDWARF -lLLVMObject -lLLVMTextAPI -lLLVMMCParser -lLLVMIRReader -lLLVMAsmParser -lLLVMMC -lLLVMDebugInfoDWARFLowLevel -lLLVMBitReader -lLLVMFrontendHLSL -lLLVMCore -lLLVMRemarks -lLLVMBitstreamReader -lLLVMBinaryFormat -lLLVMTargetParser -lLLVMSupport -lLLVMDemangle
  exit /b 0
)

if "%~1"=="--ldflags" (
  echo -L%LLVM_BUILD:\=/%/lib
  exit /b 0
)

if "%~1"=="--libnames" (
  echo libLLVMWebAssemblyDisassembler.a libLLVMMCDisassembler.a libLLVMWebAssemblyAsmParser.a libLLVMWebAssemblyCodeGen.a libLLVMWebAssemblyUtils.a libLLVMWebAssemblyDesc.a libLLVMWebAssemblyInfo.a libLLVMAsmPrinter.a libLLVMCoverage.a libLLVMLTO.a libLLVMPlugins.a libLLVMPasses.a libLLVMIRPrinter.a libLLVMHipStdPar.a libLLVMCoroutines.a libLLVMGlobalISel.a libLLVMSelectionDAG.a libLLVMCFGuard.a libLLVMExtensions.a libLLVMCodeGen.a libLLVMTarget.a libLLVMObjCARCOpts.a libLLVMCodeGenTypes.a libLLVMCGData.a libLLVMipo.a libLLVMInstrumentation.a libLLVMVectorize.a libLLVMSandboxIR.a libLLVMLinker.a libLLVMFrontendOpenMP.a libLLVMFrontendDirective.a libLLVMFrontendAtomic.a libLLVMFrontendOffloading.a libLLVMObjectYAML.a libLLVMScalarOpts.a libLLVMInstCombine.a libLLVMBitWriter.a libLLVMAggressiveInstCombine.a libLLVMTransformUtils.a libLLVMAnalysis.a libLLVMProfileData.a libLLVMSymbolize.a libLLVMDebugInfoBTF.a libLLVMDebugInfoPDB.a libLLVMDebugInfoMSF.a libLLVMDebugInfoCodeView.a libLLVMDebugInfoGSYM.a libLLVMDebugInfoDWARF.a libLLVMObject.a libLLVMTextAPI.a libLLVMMCParser.a libLLVMIRReader.a libLLVMAsmParser.a libLLVMMC.a libLLVMDebugInfoDWARFLowLevel.a libLLVMBitReader.a libLLVMFrontendHLSL.a libLLVMCore.a libLLVMRemarks.a libLLVMBitstreamReader.a libLLVMBinaryFormat.a libLLVMTargetParser.a libLLVMSupport.a libLLVMDemangle.a
  exit /b 0
)

echo unsupported target llvm-config arguments: %* 1>&2
exit /b 1
