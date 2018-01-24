# Web Bluetooth Testing

Web Bluetooth testing relies on the [Web Bluetooth Testing API] which must be
provided by browsers under test.

In this test suite `resources/bluetooth-helpers.js` detects and triggers
the API to be loaded as needed.

The Chromium implementation is provided by
`../resources/chromium/web-bluetooth-test.js`.

[Web Bluetooth Testing API]: https://docs.google.com/document/d/1Nhv_oVDCodd1pEH_jj9k8gF4rPGb_84VYaZ9IG8M_WY/

# Generated gen-* files from generator.py

`generator.py` builds `gen-*.html` tests using templates in
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