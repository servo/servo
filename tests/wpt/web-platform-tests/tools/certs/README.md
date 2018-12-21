To enable https://web-platform.test:8443/, add cacert.pem to your browser as Certificate Authority.

For Firefox, go to about:preferences and search for "certificates".

For browsers that use the Certificate Authorities of the underlying OS, such as Chrome and Safari,
you need to adjust the OS. For macOS, go to Keychain Access and add the certificate under
**login**.

### Updating these certs

From the root, run `./wpt serve --config tools/certs/config.json` and terminate it after it has started up.
