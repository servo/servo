'use strict';

test(() => {
  assert_throws_dom("NotAllowedError", () => new TCPSocket("address.com", 53), "constructor should throw");
}, "tcp disabled by permissions-policy");
test(() => {
  assert_throws_dom("NotAllowedError", () => new UDPSocket({ remoteAddress: "address.com", remotePort: 53 }), "constructor should throw");
}, "udp disabled by permissions-policy");
