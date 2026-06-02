// META: script=helper.js
//
// The following tests verify that `accept-signature` headers are sent when
// requesting resources via `<script>` elements.

let test_cases = [
  {
    integrity: `ed25519-${kValidKeys['rfc']}`,
    header: `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity"`,
  },
  {
    integrity: `ed25519-${kValidKeys['rfc']} malformed-thing`,
    header: `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity"`,
  },
  {
    integrity: `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['rfc']}`,
    header: `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity"`,
  },
  {
    integrity: `ed25519-${kValidKeys['arbitrary']} ed25519-${kValidKeys['rfc']}`,
    header: `sig0=("unencoded-digest";sf);keyid="${kValidKeys['arbitrary']}";tag="ed25519-integrity", ` +
            `sig1=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity"`,
  },
  {
    integrity: `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['arbitrary']}`,
    header: `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity", ` +
            `sig1=("unencoded-digest";sf);keyid="${kValidKeys['arbitrary']}";tag="ed25519-integrity"`,
  },
  {
    integrity: `ed25519-${kValidKeys['arbitrary']} malformed-thing ed25519-${kValidKeys['rfc']}`,
    header: `sig0=("unencoded-digest";sf);keyid="${kValidKeys['arbitrary']}";tag="ed25519-integrity", ` +
            `sig1=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity"`,
  },
  {
    integrity: `ed25519-${kValidKeys['rfc']} malformed-thing ed25519-${kValidKeys['arbitrary']}`,
    header: `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity", ` +
            `sig1=("unencoded-digest";sf);keyid="${kValidKeys['arbitrary']}";tag="ed25519-integrity"`,
  },
];

let test_counter = 0;
for (let test of test_cases) {
  test_counter++;
  async_test(t => {
    let s = document.createElement('script');
    let resource = new URL("/subresource-integrity/signatures/accept-signature-script.py", self.location);
    resource.searchParams.set("header", test.header);
    resource.searchParams.set("counter", test_counter); // Just to force independent requests.
    s.src = resource;
    s.integrity = test.integrity;
    s.onload = t.step_func_done(e => {
      assert_equals(s.getAttribute('matched'), 'true');
    });
    s.onerror = t.unreached_func("Script should not fail.");

    document.body.appendChild(s);
  }, test.integrity)
}
