## Servo on Hololens.

How to compile and run:

In your Visual Studio 2017 cmd prompt:
- compile servo: `mach build -d --uwp`

Install [HoloLens 2 Emulator](https://docs.microsoft.com/en-us/windows/mixed-reality/using-the-hololens-emulator).

With Visual Studio 2017:
- Open `support\hololens\ServoApp.sln`
- Select emulator or local machine, select configuration (Debug or Release) and press run
- VS will look for the DLLs and .h in `../../target/debug|release/` (depending on the configuration you selected in VS) and copy them in the final package.

For now, it's not possible to interact with the web page.

Note: to build the project with MSBuild:
- `MSBuild ServoApp.sln /p:Configuration=Debug /p:Platform=x64`
