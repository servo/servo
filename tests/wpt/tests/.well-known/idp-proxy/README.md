Identity Provider Proxy
=======================

This directory is used for hosting the mock identity provider proxy script
for testing the identity provider feature in WebRTC.
[[ietf-rtcweb-security-arch](https://tools.ietf.org/html/draft-ietf-rtcweb-security-arch-12#section-5.6.5)]
[[webrtc-pc](https://w3c.github.io/webrtc-pc/#sec.identity-proxy)]

The script for identity provider proxy must be hosted at /.well-known/idp-proxy
instead of the usual [/webrtc](../../webrtc) directory as it follows the
well-known URI standard that derives the script URI from a given domain name.
[[RFC5785](https://tools.ietf.org/html/rfc5785)]
