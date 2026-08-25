# WebGPU Conformance Test Suite

The WebGPU CTS requires support for the WebGPU API. This requires both browser
support and hardware support, so this API cannot run on most automated testing
infrastructure. Tests inside this directory should always be skipped if
appropriate GPU hardware is not available.

The contents of this directory are automatically generated from TypeScript
sources which live upstream in the [WebGPU CTS](https://github.com/gpuweb/cts).
They are periodically built and pushed to WPT.
