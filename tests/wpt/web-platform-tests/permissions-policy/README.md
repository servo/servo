# Permissions Policy Guide
## How to Test a New Feature with permissions policy

This directory contains a framework to test features with permissions policy.

When adding a new feature to permissions policy, the following cases should be tested:
* feature enabled by header policy [HTTP tests]
    + test when feature is enabled by permissions policy HTTP header;
* feature disabled by header policy [HTTP tests]
    + test when feature is disabled by permissions policy HTTP header;
* feature enabled on self origin by header policy [HTTP tests]
    + test when feature is enabled only on self origin by permissions policy HTTP
    header.
* feature allowed by container policy (iframe "allow" attribute);
    + test when feature is enabled by iframe "allow" attribute on self and cross
    origins.
* feature allowed by container policy, redirect on load.
    + test when feature is enabled by iframe "allow" attribute when the iframe
    is being redirected to a new origin upon loading

### How to Use the Test Framework
Use `test_feature_availability()` defined in
`/permissions-policy/resources/permissions-policy.js`. Please refer to the comments
in `/permissions-policy/resources/permissions-policy.js` for how this function works.

### How to Write Header Policy Tests
HTTP tests are used to test features with header policy.

* Define the header policy in `<feature-name>-<enabled | disabled | enabled-on-self-origin>-by-permissions-policy.https.sub.html.headers`. Example:

    Permissions-Policy: feature-name=*


* In `<feature-name>-<enabled | disabled | enabled-on-self-origin>-by-permissions-policy.https.sub.html`:
* test if feature is enabled / disabled in the main frame;
* test if feature is enabled / disabled in a same-origin iframe;
* test if feature is enabled / disabled in a cross-origin iframe.

Examples:
`/permissions-policy/payment-disabled-by-permissions-policy.https.sub.html`
`/permissions-policy/payment-disabled-by-permissions-policy.https.sub.html.headers`

### How to Write Container Policy Tests
Simply use `test_feature_availability()` with the optional argument
`feature_name` specified to test if:
* feature is enabled / disabled in a same-origin iframe;
* feature is enabled / disabled in a cross-origin iframe.

Example:
`/permissions-policy/payment-allowed-by-permissions-policy-attribute.https.sub.html`

### How to Write Container Policy Tests with Redirect
Similar to the section above, append
`/permissions-policy/resources/redirect-on-load.html#` to the argument `src`
passed to `test_feature_availability()`.

Example:
`/permissions-policy/payment-allowed-by-permissions-policy-attribute-redirect-on-load.https.sub.html`

