choco install curl
curl -sSf http://releases.llvm.org/%LLVM_VERSION%/LLVM-%LLVM_VERSION%-win32.exe -o LLVM.exe
7z x LLVM.exe -oC:\LLVM
set PATH=%PATH%;C:\LLVM\bin
set LIBCLANG_PATH=C:\LLVM\bin