# Web Bluetooth Testing

Web Bluetooth testing relies on the [Web Bluetooth Testing API] which must be
provided by browsers under test.

In this test suite `resources/bluetooth-helpers.js` detects and triggers
the API to be loaded as needed.

The Chromium implementation is provided by
`../resources/chromium/web-bluetooth-test.js`.

The Chromium implementation is not included in stable Chrome builds since it
would add too much to the binary size. On Chromium infrastructure, it is run
using the `content_shell` executable.

In the future, Chromium `src/device/bluetooth` may be refactored into a Mojo
service. At this point, it would be possible to add the necessary testing hooks
into stable Chrome without substantially increasing the binary size, similar to
WebUSB.

These bluetooth tests are upstreamed here because other browsers can reuse them
by implementing the [Web Bluetooth Testing API], even if only on their internal
infrastructure.

[Web Bluetooth Testing API]: https://docs.google.com/document/d/1Nhv_oVDCodd1pEH_jj9k8gF4rPGb_84VYaZ9IG8M_WY/

# Generated gen-* files from generate.py

`generate.py` builds `gen-*.html` tests using templates in
`script-tests/*/*.js`.

The subdirectory structure in `bluetooth/script-test/*` is recreated into
`bluetooth/*`.  The generator expands each CALL function from templates
into new leaf directories and files.

Example:

`script-tests/server/get-same-object.js` contains:

```
gattServer.CALLS([
        getPrimaryService('heart_rate')|
        getPrimaryServices()|
        getPrimaryServices('heart_rate')[UUID]]),
```

Generating:

```
server/getPrimaryService/gen-get-same-object.html
server/getPrimaryServices/gen-get-same-object.html
server/getPrimaryServices/gen-get-same-object-with-uuid.html
```

Usage:

```
$ python generate.py
```

More details documented in `generate.py`.
