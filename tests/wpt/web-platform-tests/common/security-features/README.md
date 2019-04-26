This directory contains the common infrastructure for the following tests.
- referrer-policy/
- mixed-content/
- upgrade-insecure-requests/

Subdirectories:
- `subresource`:
    Serves subresources, with support for redirects, stash, etc.
    The subresource paths are managed by `subresourceMap` and
    fetched in `requestVia*()` functions in `resources/common.js`.
- `scope`:
    Serves nested contexts, such as iframe documents or workers.
    Used from `invokeFrom*()` functions in `resources/common.js`.
