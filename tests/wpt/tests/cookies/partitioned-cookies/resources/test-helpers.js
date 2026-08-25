// Test that a partitioned cookie set by |origin| with name |cookieName| is
// or is not sent in a request to |origin|.
//
// If |expectsCookie| is true, then the test cookie should be present in the
// request.
function testHttpPartitionedCookies({origin, cookieNames, expectsCookie}) {
  promise_test(async () => {
    const resp = await credFetch(`${origin}/cookies/resources/list.py`);
    const cookies = await resp.json();
    for (const cookieName of cookieNames) {
      assert_equals(
          cookies.hasOwnProperty(cookieName), expectsCookie,
          getPartitionedCookieAssertDesc(expectsCookie, cookieName));
    }
  }, getPartitionedCookieTestName(expectsCookie, "HTTP"));
}

function getPartitionedCookieTestName(expectsCookie, cookieType) {
  if (expectsCookie) {
    return "Partitioned cookies accessible on the top-level site they are " +
        `created in via ${cookieType}`;
  }
  return "Partitioned cookies are not accessible on a different top-level " +
      `site via ${cookieType}`;
}

function getPartitionedCookieAssertDesc(expectsCookie, cookieName) {
  if (expectsCookie) {
    return `Expected ${cookieName} to be available on the top-level site it ` +
        "was created in";
  }
  return `Expected ${cookieName} to not be available on a different ` +
      "top-level site";
}

function testDomPartitionedCookies({cookieNames, expectsCookie}) {
  test(() => {
    for (const cookieName of cookieNames) {
      assert_equals(
          document.cookie.includes(cookieName), expectsCookie,
          getPartitionedCookieAssertDesc(expectsCookie, cookieName));
    }
  }, getPartitionedCookieTestName(expectsCookie, "DOM"));
}

function testCookieStorePartitionedCookies({cookieNames, expectsCookie}) {
  if (!window.cookieStore) return;
  promise_test(async () => {
    const cookies = await cookieStore.getAll({partitioned: true});
    for (const cookieName of cookieNames) {
      assert_equals(
          !!cookies.find(c => c.name === cookieName), expectsCookie,
          getPartitionedCookieAssertDesc(expectsCookie, cookieName));
    }
  }, getPartitionedCookieTestName(expectsCookie, "CookieStore"));
}

function getCookieNames() {
  const cookieNames = ["__Host-pchttp", "__Host-pcdom"];
  if (window.cookieStore) {
    cookieNames.push("__Host-pccookiestore");
  }
  return cookieNames;
}
