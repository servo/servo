---
layout: page
title: Chrome for Android
---
To run WPT on Chrome on an Android device, some additional set up is required.

First of all, as usual Android development, we need to have `adb` and be able to
connect to the device.

## Hosts

Until we find a better way, we need to root the Android device and update the
/etc/hosts file to include

```
127.0.0.1   web-platform.test
127.0.0.1   www.web-platform.test
127.0.0.1   www1.web-platform.test
127.0.0.1   www2.web-platform.test
127.0.0.1   xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1   xn--lve-6lad.web-platform.test
0.0.0.0     nonexistent-origin.web-platform.test
```

## CA certificate

In order to run HTTPS tests, we need to add
[WPT's CA](https://github.com/w3c/web-platform-tests/blob/master/tools/certs/cacert.pem)
to the phone. First, convert the certificate from PEM to CRT:

```
openssl x509 -outform der -in tools/certs/cacert.pem -out cacert.crt
```

Then copy `cacert.crt` to your phone's external storage (preferably to
Downloads/ as it'll be easier to find). Open Settings -> Security & location ->
Encryption & credentials -> Install from storage. Find and install `cacert.crt`.
(The setting entries might be slightly different based your Android version.)


Finally, we may run wpt with the `chrome_android` product

```
./wpt run chrome_android [test_list]
```
