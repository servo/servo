// Only this test uses the handlers in this directory, so the credentials that
// the user agent stores for it don't affect other tests.
const directory = "/fetch/api/resources/stored-credentials/";
const redirectURL = "/common/redirect.py?location=" + encodeURIComponent(directory + "dump-authorization-header.py");

promise_setup(() => new Promise((resolve, reject) => {
  const xhr = new XMLHttpRequest();
  xhr.open("GET", directory + "authentication.py", true, "user", "password");
  xhr.onload = () => xhr.status === 200 ? resolve() : reject(new Error(`Unexpected status ${xhr.status}`));
  xhr.onerror = () => reject(new Error("Network error"));
  xhr.send();
}));

promise_test(async () => {
  const response = await fetch(redirectURL, { credentials: "include" });
  assert_implements_optional(await response.text() === "Basic " + btoa("user:password"), "the user agent stored the credentials");
}, "Stored credentials are sent after a same-origin redirect with credentials \"include\"");

promise_test(async () => {
  const response = await fetch(redirectURL, { credentials: "omit" });
  assert_equals(await response.text(), "none");
}, "Stored credentials are not sent after a same-origin redirect with credentials \"omit\"");
