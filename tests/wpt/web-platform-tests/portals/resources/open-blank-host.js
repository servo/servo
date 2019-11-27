// Portal tests often need to create portals in a context other than the one
// in which the tests are running. This is because the host context may be
// discarded during the course of the test.

// Opens a blank page for use as a portal host.
// Tests cannot simply use window.open() without a URL as about:blank may not
// host a portal.
async function openBlankPortalHost() {
  let hostWindow = window.open('/portals/resources/blank-host.html');
  await new Promise((resolve) => {
    hostWindow.addEventListener('load', resolve, {once: true});
  });
  return hostWindow;
}
