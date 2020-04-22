set destdir=%1
set cachedir=%1.%PYTHON_ARCH%

for %%d in (openssl %destdir%) do (
    if exist %%d (
        rmdir /s /q %%d
    )
)

if %PYTHON_ARCH% == 64 (
   set OPENSSL_CONFIG=VC-WIN64A
   set VC_ARCH=x64
) else (
   set OPENSSL_CONFIG=VC-WIN32
   set VC_ARCH=x86
)

call "C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvarsall.bat" %VC_ARCH%
SET PATH=%PATH%;C:\Program Files\NASM

if not exist %cachedir% (
    mkdir openssl
    curl -L https://www.openssl.org/source/openssl-1.1.1f.tar.gz -o openssl.tar.gz
    tar xzf openssl.tar.gz -C openssl --strip-components 1
    cd openssl

    perl Configure no-comp no-shared no-tests %OPENSSL_CONFIG%
    nmake

    mkdir %cachedir%
    mkdir %cachedir%\include
    mkdir %cachedir%\lib
    xcopy include %cachedir%\include\ /E
    copy libcrypto.lib %cachedir%\lib\
    copy libssl.lib %cachedir%\lib\
)

mkdir %destdir%
xcopy %cachedir% %destdir% /E
