/**
 * Test cases based on the examples listed in the API proposal:
 * https://github.com/w3c/webextensions/blob/main/proposals/public-suffix.md
 */

function testPublicSuffixAPIs(hostname, expectedDomain, expectedKnownSuffix, description) {
  browser.test.assertEq(
    browser.publicSuffix.getDomain(hostname),
    expectedDomain,
    description + " (getDomain)"
  );

  browser.test.assertEq(
    browser.publicSuffix.isKnownSuffix(hostname),
    !!expectedKnownSuffix && !expectedDomain,
    description + " (isKnownSuffix)"
  );

  browser.test.assertEq(
    browser.publicSuffix.getKnownSuffix(hostname),
    expectedKnownSuffix,
    description + " (getKnownSuffix)"
  );
}

browser.test.runTests([
  async function testInvalidHostnames() {
    const testCases = [
      ["*.com", "Contains invalid character '*'"],
      ["", "Empty string"],
      [".", "No domain labels"],
      ["example..com", "Contains empty domain label"],
      ["example。.com", "Contains empty domain label (Unicode dot)"],
      [".example.com", "Dot in front"],
      ["..example.com", "Two dots in front"],
      ["example.net/path", "Contains a path"],
      ["example.net:80", "Contains a port"],
      ["user@example.net", "Contains credentials"],
      ["https://example.net", "Full URL"],
      ["example .net", "Contains whitespace"],
      ["example.net..", "Two dots at the end"],
      ["sub%2eexample.net", "Contains percent encoding"],
      ["::1", "IPv6 address without brackets"],
      [".127.0.0.1", "Dot before an IPv4 address"],
      [".[::1]", "Dot before an IPv6 address"]
    ];

    for (const [hostname, description] of testCases) {
      for (const method of ["isKnownSuffix", "getDomain", "getKnownSuffix"]) {
        browser.test.assertThrows(
          () => {
            browser.publicSuffix[method](hostname);
          },
          /.*/, // Varies per platform, e.g. `Invalid hostname: ${hostname}`.
          `${description} (${method})`
        );
      }
    }
  },

  async function testHostnameNormalization() {
    testPublicSuffixAPIs(
      "foo_bar.example.net",
      "example.net",
      "net",
      "Hostname contains underscore"
    );

    testPublicSuffixAPIs(
      "example。net",
      "example.net",
      "net",
      "Unicode dot in hostname"
    );

    testPublicSuffixAPIs(
      "EXAMPLE.NET",
      "example.net",
      "net",
      "Uppercase hostname"
    );

    testPublicSuffixAPIs(
      "example.net.",
      "example.net.",
      "net.",
      "Trailing dot in hostname with known suffix"
    );

    testPublicSuffixAPIs(
      "example.invalid.",
      null,
      null,
      "Trailing dot in hostname with unknown suffix"
    );

    testPublicSuffixAPIs(
      "foo.xn--mgbx4cd0ab",
      "foo.xn--mgbx4cd0ab",
      "xn--mgbx4cd0ab",
      "Punycode hostname"
    );

    testPublicSuffixAPIs(
      "foo.مليسيا",
      "foo.xn--mgbx4cd0ab",
      "xn--mgbx4cd0ab",
      "Unicode hostname"
    );
  },

  async function testGetDomainOptions() {
    // allowPlainSuffix
    browser.test.assertEq(
      browser.publicSuffix.getDomain("net"),
      null,
      "Is an eTLD itself, single-label"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("net", { allowPlainSuffix: false }),
      null,
      "Is an eTLD itself, single-label, with allowPlainSuffix = false"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("net", { allowPlainSuffix: true }),
      "net",
      "Is an eTLD itself, single-label, with allowPlainSuffix = true"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("github.io"),
      null,
      "Is an eTLD itself, multi-label, in the private section"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("github.io", { allowPlainSuffix: false }),
      null,
      "Is an eTLD itself, multi-label, in the private section, with allowPlainSuffix = false"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("github.io", { allowPlainSuffix: true }),
      "github.io",
      "Is an eTLD itself, multi-label, in the private section, with allowPlainSuffix = true"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("co.uk", { allowPlainSuffix: true }),
      "co.uk",
      "Multi-label ICANN suffix with allowPlainSuffix = true"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("ck", { allowPlainSuffix: true }),
      "ck",
      "Wildcard suffix with allowPlainSuffix = true"
    );

    // allowUnknownSuffix
    browser.test.assertEq(
      browser.publicSuffix.getDomain("invalid"),
      null,
      "No matching eTLD in PSL, single-label"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("invalid", {
        allowUnknownSuffix: false
      }),
      null,
      "No matching eTLD in PSL, single-label, with allowUnknownSuffix = false"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("invalid", {
        allowUnknownSuffix: true
      }),
      "invalid",
      "No matching eTLD in PSL, single-label, with allowUnknownSuffix = true"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("my.net.invalid", {
        allowUnknownSuffix: false
      }),
      null,
      "No matching eTLD in PSL, multi-label, with allowUnknownSuffix = false"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("my.net.invalid", {
        allowUnknownSuffix: true
      }),
      "net.invalid",
      "No matching eTLD in PSL, multi-label, with allowUnknownSuffix = true"
    );

    // allowPlainSuffix + allowUnknownSuffix
    browser.test.assertEq(
      browser.publicSuffix.getDomain("invalid", {
        allowUnknownSuffix: false,
        allowPlainSuffix: true
      }),
      null,
      "No matching eTLD in PSL, single-label, with allowUnknownSuffix = false and allowPlainSuffix = true"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("invalid", {
        allowUnknownSuffix: true,
        allowPlainSuffix: false
      }),
      "invalid",
      "No matching eTLD in PSL, single-label, with allowUnknownSuffix = true and allowPlainSuffix = false"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("my.net.invalid", {
        allowUnknownSuffix: false,
        allowPlainSuffix: true
      }),
      null,
      "No matching eTLD in PSL, multi-label, with allowUnknownSuffix = false and allowPlainSuffix = true"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("my.net.invalid", {
        allowUnknownSuffix: true,
        allowPlainSuffix: false
      }),
      "net.invalid",
      "No matching eTLD in PSL, multi-label, with allowUnknownSuffix = true and allowPlainSuffix = false"
    );

    // encoding
    browser.test.assertEq(
      browser.publicSuffix.getDomain("foo.مليسيا"),
      "foo.xn--mgbx4cd0ab",
      "This is an IDN"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("foo.مليسيا", { encoding: "display" }),
      "foo.مليسيا",
      "This is an IDN, with encoding = display"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("foo.مليسيا", { encoding: "punycode" }),
      "foo.xn--mgbx4cd0ab",
      "This is an IDN, with encoding = punycode"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("foo.xn--mgbx4cd0ab"),
      "foo.xn--mgbx4cd0ab",
      "This is an IDN, but Punycode"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("foo.xn--mgbx4cd0ab", {
        encoding: "display"
      }),
      "foo.مليسيا",
      "This is an IDN, but Punycode, with encoding = display"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("xn--bs-red.com", { encoding: "display" }),
      "xn--bs-red.com",
      "bսs.com in Unicode, kept as punycode even with encoding = display due to presence of confusable"
    );

    // Note: allowIPAddress option tested below.
  },

  async function testIPAddresses() {
    testPublicSuffixAPIs("127.0.0.1", null, null, "IPv4 address");
    testPublicSuffixAPIs("[::1]", null, null, "IPv6 address");

    browser.test.assertEq(
      browser.publicSuffix.getDomain("127.0.0.1", { allowIPAddress: true }),
      "127.0.0.1",
      "IP address, IPv4, with allowIPAddress = true"
    );

    browser.test.assertEq(
      browser.publicSuffix.getDomain("[::1]", { allowIPAddress: true }),
      "[::1]",
      "IPv6 address, with allowIPAddress = true"
    );
  },


  async function testKnownSuffixes() {
    testPublicSuffixAPIs(
      "net",
      null,
      "net",
      "Known single-label ICANN suffix"
    );

    testPublicSuffixAPIs(
      "co.uk",
      null,
      "co.uk",
      "Known multi-label ICANN suffix"
    );

    testPublicSuffixAPIs(
      "github.io",
      null,
      "github.io",
      "Known private suffix"
    );

    testPublicSuffixAPIs(
      "مليسيا",
      null,
      "xn--mgbx4cd0ab",
      "Known Unicode suffix"
    );

    testPublicSuffixAPIs(
      "xn--mgbx4cd0ab",
      null,
      "xn--mgbx4cd0ab",
      "Known Punycode suffix"
    );
  },

  async function testRegistrableDomains() {
    testPublicSuffixAPIs(
      "example.net",
      "example.net",
      "net",
      "Registrable domain with a single-label ICANN suffix"
    );

    testPublicSuffixAPIs(
      "www.example.net",
      "example.net",
      "net",
      "Subdomain of a registrable domain with a single-label ICANN suffix"
    );

    testPublicSuffixAPIs(
      "sub.domain.co.uk",
      "domain.co.uk",
      "co.uk",
      "Subdomain of a registrable domain with a multi-label ICANN suffix"
    );

    testPublicSuffixAPIs(
      "foobar.github.io",
      "foobar.github.io",
      "github.io",
      "Registrable domain with a private suffix"
    );

    testPublicSuffixAPIs(
      "sub.foobar.github.io",
      "foobar.github.io",
      "github.io",
      "Subdomain of a registrable domain with a private suffix"
    );
  },

  async function testUnknownSuffixes() {
    testPublicSuffixAPIs(
      "invalid",
      null,
      null,
      "Single-label hostname with an unknown suffix"
    );

    testPublicSuffixAPIs(
      "my.net.invalid",
      null,
      null,
      "Multi-label hostname with an unknown suffix"
    );
  },

  async function testWildcardSuffixes() {
    testPublicSuffixAPIs(
      "ck",
      null,
      "ck",
      "Base of a wildcard suffix rule"
    );

    testPublicSuffixAPIs(
      "any.ck",
      null,
      "any.ck",
      "Suffix matching a wildcard rule"
    );

    testPublicSuffixAPIs(
      "sub.domain.any.ck",
      "domain.any.ck",
      "any.ck",
      "Subdomain of a registrable domain with a wildcard suffix"
    );

    testPublicSuffixAPIs(
      "sub.sub.domain.any.ck",
      "domain.any.ck",
      "any.ck",
      "Multiple subdomain labels above a registrable domain with a wildcard suffix"
    );
  },

  async function testExceptionSuffixes() {
    testPublicSuffixAPIs(
      "www.ck",
      "www.ck",
      "ck",
      "Exception to a wildcard suffix rule"
    );

    testPublicSuffixAPIs(
      "sub.www.ck",
      "www.ck",
      "ck",
      "Subdomain of a wildcard exception"
    );

    testPublicSuffixAPIs(
      "sub.sub.www.ck",
      "www.ck",
      "ck",
      "Multiple subdomain labels above a wildcard exception"
    );
  },
]);
