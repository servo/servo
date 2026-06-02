# Web Bluetooth API Tests

The Web Bluetooth API enables sites to connect to and interact with Bluetooth
Low Energy devices. Please check the [Web Bluetooth specification] for more
details.

Web Bluetooth testing relies on the [FakeBluetooth][Web Bluetooth
Testing] test API which must be provided by browsers under test.

TODO([#485]): Update the links to [FakeBluetooth][Web Bluetooth Testing] to
point to the [Testing Web Bluetooth specification].

In this test suite `resources/bluetooth-test.js` detects and triggers
the API to be loaded as needed. This file also contains test helper methods,
such as for asserting that Bluetooth events are fired in a specific order.
The `resources/bluetooth-fake-devices.js` contains several helper methods that set
up fake Bluetooth devices.

[Web Bluetooth specification]: https://WebBluetoothCG.github.io/web-bluetooth
[Web Bluetooth Testing]:
https://docs.google.com/document/d/1Nhv_oVDCodd1pEH_jj9k8gF4rPGb_84VYaZ9IG8M_WY/
[#485]: https://github.com/WebBluetoothCG/web-bluetooth/issues/485
[Testing Web Bluetooth specification]:
https://WebBluetoothCG.github.io/web-bluetooth/tests.html

## Generated Tests

Several Web Bluetooth tests share common test logic. For these tests, the
`script-tests` directory contains templates that are used by the
`generate.py` script to create several tests from these templates. The templates
are JavaScript files that contain a `CALLS()` keyword with functions delimited by
a `|` character. A test will be created for each function in the `CALLS()` by
`generate.py`. Note that for each subdirectory in `script-tests` there is a
matching directory under `bluetooth`. The generator will expand `CALLS`
functions into the
corresponding directory.

### Example

The `./script-tests/server/get-same-object.js` contains the following
code:

```js
gattServer.CALLS([
        getPrimaryService('heart_rate')|
        getPrimaryServices()|
        getPrimaryServices('heart_rate')[UUID]]),
```

The functions in `CALLS()` will be expanded to generate 3 test files prefixed
with `gen-`:

```
bluetooth/server/getPrimaryService/gen-get-same-object.html
bluetooth/server/getPrimaryServices/gen-get-same-object.html
bluetooth/server/getPrimaryServices/gen-get-same-object-with-uuid.html
```

### Generate Tests

To generate the tests in `script-tests`, run the following command from the
source root:

```sh
$ python bluetooth/generate.py
```

To check that generated tests are correct and that there are no obsolete tests,
or tests for which a template does not exist anymore, run:

```sh
$ python bluetooth/generate_test.py
```

More details can be found in `generate.py` and `generate_test.py`.

## Chromium Implementation
The Chromium implementation is provided by
`../resources/chromium/web-bluetooth-test.js` using [MojoJS].

The Chromium implementation is not included in stable Chrome builds since it
would add too much to the binary size. On Chromium infrastructure, it is run
using the `content_shell` executable.

In the future, Chromium `src/device/bluetooth` may be refactored into a Mojo
service. At this point, it would be possible to add the necessary testing hooks
into stable Chrome without substantially increasing the binary size, similar to
WebUSB.

These Bluetooth tests are upstreamed here because other browsers can reuse them
by implementing the [Web Bluetooth Testing] API, even if only on their internal
infrastructure.

For more implementation details, see the [Web Bluetooth Service README].

[MojoJS]: https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/testing/web_platform_tests.md#mojojs
[Web Bluetooth Service README]:
https://chromium.googlesource.com/chromium/src.git/+/main/content/browser/bluetooth/README.md

# Resources and Documentation

For any issues pertaining to the specification, please file a [GitHub]
issue. For issues pertaining to an implementation of Web Bluetooth, please
file an issue with the implementor's bug tracker.

* [Web Bluetooth specification]
* [Web Bluetooth Testing]
* [Testing Web Bluetooth specification]

[GitHub]: https://github.com/WebBluetoothCG/web-bluetooth

## Chromium

Mailing list: web-bluetooth@chromium.org

Bug tracker: [Blink>Bluetooth]

* [Web Bluetooth Service README]

[Blink>Bluetooth]: https://bugs.chromium.org/p/chromium/issues/list?q=component%3ABlink%3EBluetooth&can=2
