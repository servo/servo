---
layout: page
title: Chrome for Android
---
To run WPT on Chrome on an Android device, some additional set up is required.

First of all, as usual Android development, we need to have `adb` and be able to
connect to the device.

## Hosts

Until we find a better way, we need to root the Android device and update the
/etc/hosts file to include the entries printed by `./wpt make-hosts-file`.

## CA certificate

In order to run HTTPS tests, we need to add
[WPT's CA](https://github.com/web-platform-tests/wpt/blob/master/tools/certs/cacert.pem)
to the phone. First, convert the certificate from PEM to CRT:

```
openssl x509 -outform der -in tools/certs/cacert.pem -out cacert.crt
```

Then copy `cacert.crt` to your phone's external storage (preferably to
Downloads/ as it'll be easier to find). Open Settings -> Security & location ->
Encryption & credentials -> Install from storage. Find and install `cacert.crt`.
(The setting entries might be slightly different based your Android version.)

Note that having this CA installed on your device outside of a test
environment represents a security risk.


Finally, we may run wpt with the `chrome_android` product

```
./wpt run chrome_android [test_list]
```
