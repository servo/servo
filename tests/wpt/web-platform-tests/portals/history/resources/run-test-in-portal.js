// This is called from the portal host which is running with the test harness.
// This creates a portal and communicates with our ad hoc test harness in the
// portal context which performs the history manipulation in the portal. We
// confirm that the history manipulation works as expected in the portal.
async function runTestInPortal(portalSrc, testName) {
  let portal = document.createElement('portal');
  portal.src = portalSrc + '?testName=' + testName;
  let result = await new Promise((resolve) => {
    portal.onmessage = (e) => {
      resolve(e.data);
    };
    document.body.appendChild(portal);
  });

  assert_equals(result, 'Passed');
}
