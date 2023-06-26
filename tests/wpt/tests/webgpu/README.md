# WebGPU Conformance Test Suite

The WebGPU CTS requires support for the WebGPU API. This requires both browser
support and hardware support, so this API cannot run on most automated testing
infrastructure. Tests inside this directory should always be skipped if
appropriate GPU hardware is not available.

~~The contents of this directory are automatically generated from TypeScript
sources which live upstream in the [WebGPU CTS](https://github.com/gpuweb/cts).
They are periodically built and pushed to WPT.~~

**NOTE:** This directory is currently empty while WebGPU is in flux, as it's
not practical for browser developers to sync their WebGPU implementations with
WPT auto-import and auto-export of these tests. Instead, browsers should pin
("vendor") a specific version of the gpuweb/cts repository, and check the built
files into the browser repository as non-exported wpt tests (like Chromium's
`wpt_internal`).
