These tests exercise different ways to fetch a resource (image, font-face, svg
references), generated via the sub-resource python script in
```./generic/subresource/``` (for example, loading an image:
```/common/security-features/subresource/image.py?id=<UUID>```) and later verify
the headers used to fetch the resource.

Since there is no way to wait for a resource referenced from CSS to be loaded,
all tests use ```step_timeout()``` to delay the verification step until
after the resource (hopefully) was loaded.

Since there is also no way to retrieve headers (or other information) from
resources loaded via CSS, we store the headers with the given ```UUID``` as key
on the server, and retrieve them later via an XHR, for example:
```/common/security-features/subresource/image.py?id=<UUID>&report-headers```.
