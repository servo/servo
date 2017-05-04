These tests exercise differnt ways to load an image, generated via
```/referrer-policy/generic/subresource/image.py?id=<UUID>``` and later
verify the headers used to request that image.

Since there is no way to wait for a resource referenced from CSS to be loaded,
all tests use ```step_timeout()``` to delay the verification step until
after the image (hopefully) was loaded.

Since there is also no way to retrieve headers (or other information) from
images loaded via CSS, we store the headers with the given ```UUID``` as key
on the server, and retrieve them later via an XHR to
```/referrer-policy/generic/subresource/image.py?id=<UUID>&report-headers```.
