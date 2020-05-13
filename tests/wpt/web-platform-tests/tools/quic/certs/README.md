To generate cert.key and cert.pem:

 1. Remove web-platform.test.key and web-platform.test.pem in ../../certs.
 1. From the root, run
    `./wpt serve --config tools/quic/certs/config.json` and terminate it
    after it has started up.
 1. Move tools/certs/web-platform.test.key to tools/quic/certs/cert.key.
 1. Move tools/certs/web-platform.test.pem to tools/quic/certs/cert.pem.
 1. Recover the original web-platform.test.key and web-platform.test.pem in
    ../../certs.

See also: ../../certs/README.md