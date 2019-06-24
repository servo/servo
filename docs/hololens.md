## Servo on Hololens.

How to compile and run:

With Visual Studio **2019**:
- Open `support/hololens/ServoApp.sln`
- click on *restore nugets packages* under the context menu of "Solution" (in the right panel). This will automatically download Angle which comes with libEGL.dll, necessary to build servo.

In your Visual Studio **2017** cmd prompt:
- make sure libEGL.dll is in your `%LIB%` path: `set LIB=%LIB%;c:\XXX\servo\support\hololens\packages\ANGLE.WindowsStore.2.1.13\bin\UAP\x64\`
- compile servo: `mach build -d --libsimpleservo --features raqote_backend no_wgl`

With Visual Studio **2019**:
- Select emulator or local machine, select configuration (Debug or Release) and press run
- VS will look for the DLLs and .h in `../../target/debug|release/` (depending on the configuration you selected in VS) and copy them in the final package.

For now, it's not possible to interact with the web page.

Note: to build the project with MSBuild:
- `MSBuild ServoApp.sln /p:Configuration=Debug /p:Platform=x64`
