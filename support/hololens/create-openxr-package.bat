REM USAGE
REM Clone https://github.com/microsoft/OpenXR-SDK-VisualStudio, open the openxr_loader_uwp project
REM Change the project output type to a dynamic library
REM Build it for Debug/Release x64/ARM64
REM create-openxr-package path\to\outputdir path\to\OpenXR-SDK-VisualStudio
REM name the outputdir openxr-loader-uwp-versionnumber and zip it

cd %1
mkdir arm64
mkdir arm64\Debug
cd arm64\Debug
copy %2\bin\Debug\ARM64\openxr_loader_uwp\* .
ren *.* openxr_loader.*
cd ..
mkdir Release
cd Release
copy %2\bin\Release\ARM64\openxr_loader_uwp\* .
ren *.* openxr_loader.*
cd ..\..
mkdir x64
mkdir x64\Debug
cd x64\Debug
copy %2\bin\Debug\x64\openxr_loader_uwp\* .
ren *.* openxr_loader.*
cd ..
mkdir Release
cd Release
copy %2\bin\Release\x64\openxr_loader_uwp\* .
ren *.* openxr_loader.*