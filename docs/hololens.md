# Using Servo on HoloLens 2

To build Servo for the HoloLens 2, see [the wiki](https://github.com/servo/servo/wiki/Building-for-UWP).

## Web development workflow

#### To load a page without typing on the on-screen keyboard:
1. ensure Firefox Reality is installed
1. create a QR code for `fxr://[url]` (eg. https://www.qr-code-generator.com/)
1. display it on some visible screen or surface (relatively large)
1. point the HoloLens towards the QR code, wait until it is recognised
1. accept the prompt to open the URL in Firefox Reality

#### Use URL shorteners to minimize typing

Services like [https://free-url-shortener.rb.gy/](rb.gy) can generate short URLs like https://rb.gy/kqxz7h,
so you only need to type `rb.gy/kqxz7h` on the on-screen keyboard.

#### Viewing JS output and exceptions

1. Press the "developer" button to the right of the URL bar in Firefox Reality
1. A panel appears at the bottom of the app which acts as a JS console for the current page.

#### Viewing JS output and exceptions in immersive mode

Before entering immersive mode, use the desktop Firefox [remote developer tools](https://github.com/servo/servo/wiki/Devtools)
to access the Firefox Reality JS console.

## Exiting immersive mode

#### Pausing immersive mode

Use the default "bloom gesture" (two fingers on a wrist) to pause immersive mode and return to the home environment.
Tap the Firefox Reality app to re-enter immersive mode and continue interacting with web content.

#### Exiting immersive mode

Use the "palm gesture" - hold your palm parallel to your face for several seconds. A prompt will appear which
will allow you to cancel the prompt exit or exit immersive mode and return to the 2d browsing session.

#### Exiting immersive mode via web content

If your content can respond to user interaction events, invoking the [XrSession.end](https://immersive-web.github.io/webxr/#dom-xrsession-end)
API will exit immersive mode and return to 2d browsing.


## Reporting issues

If your web content is not rendering the same as other browsers, or is throwing exceptions that only appear in Firefox Reality,
please [file an issue](https://github.com/servo/servo/issues/new)! When possible, please provide a link that demonstrates the problem.

It can be useful to include application logs in the issue report. To find the log of the most recent Firefox Reality session,
open the [Device Portal](http://127.0.0.1:10080/), go to `System` -> `File explorer` -> `LocalAppData` -> `Firefox Reality` -> `AppData`,
and attach the `stdout.txt` file that is present in that directory.
