'use strict';

// A test function that runs the common steps for requesting an XR session.
// After the session is created, it is initialize the XRWebGLBinding
// and local XRSpace objects for the session. These components are essential
// for tests involving WebXR layers.
function xr_layer_promise_test(
    name, func, fakeDeviceInit, sessionMode, sessionInit, properties,
    glcontextPropertiesParam) {
  const glcontextProperties = (glcontextPropertiesParam) ? glcontextPropertiesParam : {};

  function runTest(t, glContext) {
    let testSession;
    let testDeviceController;
    let sessionObjects = {gl: glContext};

    // Ensure that any pending sessions are ended when done. This needs to
    // use a cleanup function to ensure proper sequencing. If this were
    // done in a .then() for the success case, a test that expected
    // failure would already be marked done at the time that runs, and the
    // shutdown would interfere with the next test which may have started.
    t.add_cleanup(async () => {
      // If a session was created, end it.
      if (testSession) {
        await testSession.end().catch(() => {});
      }
    });

    return navigator.xr.test.simulateDeviceConnection(fakeDeviceInit)
        .then((controller) => {
          testDeviceController = controller;
          return sessionObjects.gl.makeXRCompatible();
        })
        .then(() => new Promise((resolve, reject) => {
                // Perform the session request in a user gesture.
                xr_debug(name, 'simulateUserActivation');
                navigator.xr.test.simulateUserActivation(() => {
                  xr_debug(name, 'document.hasFocus()=' + document.hasFocus());
                  navigator.xr.requestSession(sessionMode, sessionInit || {})
                      .then(async (session) => {
                        xr_debug(name, 'session start');
                        testSession = session;
                        session.mode = sessionMode;
                        session.sessionInit = sessionInit;
                        // This method creates test specific session objects.
                        sessionObjects.xrBinding = new XRWebGLBinding(session, sessionObjects.gl);
                        // Request a 'local' reference space which is required for layers creation.
                        sessionObjects.xrSpace = await session.requestReferenceSpace('local');
                        if (!sessionObjects.xrSpace) {
                          reject("Local space is required for layers test.");
                          return;
                        }
                        xr_debug(name, 'session.visibilityState=' + session.visibilityState);
                        try {
                          resolve(func(session, testDeviceController, t, sessionObjects));
                        } catch(err) {
                          reject("Test function failed with: " + err);
                        }
                      })
                      .catch((err) => {
                        xr_debug(name, 'error: ' + err);
                        reject(
                            'Session with params ' +
                            JSON.stringify(sessionMode) +
                            ' was rejected on device ' +
                            JSON.stringify(fakeDeviceInit) +
                            ' with error: ' + err);
                      });
                });
        }));
  }

  xr_promise_test(
    name + ' - webgl',
    runTest,
    properties,
    'webgl',
    {alpha: false, antialias: false, ...glcontextProperties}
  );
  xr_promise_test(
    name + ' - webgl2',
    runTest,
    properties,
    'webgl2',
    {alpha: false, antialias: false, ...glcontextProperties}
  );
}
