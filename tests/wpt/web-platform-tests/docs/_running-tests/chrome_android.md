To run WPT on Chrome on an android device, some additional set up is required.
First until we find a better way, we need to root the android device and update
the /etc/hosts file to include

```
127.0.0.1   web-platform.test
127.0.0.1   www.web-platform.test
127.0.0.1   www1.web-platform.test
127.0.0.1   www2.web-platform.test
127.0.0.1   xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1   xn--lve-6lad.web-platform.test
0.0.0.0     nonexistent-origin.web-platform.test
```

Next, we need to start adb and reverse forward the web-platform.tests ports

```
adb start-server
```

Add any ports that you need based on your config. For example:
```
adb reverse tcp:8000 tcp:8000
adb reverse tcp:8001 tcp:8001
adb reverse tcp:8081 tcp:8081
adb reverse tcp:8444 tcp:8444
adb reverse tcp:9001 tcp:9001
adb reverse tcp:9444 tcp:9444
```

After this, we may run wpt with the `chrome_android` product

```
./wpt run chrome_android <test>
```
