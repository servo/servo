function runTest(config, qualifier)
{
  function checkStatusForMinHdcpVersionPolicy(hdcpVersion)
  {
    return navigator.requestMediaKeySystemAccess(config.keysystem, getSimpleConfiguration())
        .then(function(access) {
          return access.createMediaKeys();
        })
        .then(function(mediaKeys) {
          // As HDCP policy depends on the hardware running this test,
          // don't bother checking the result returned as it may or
          // may not be supported. This simply verifies that
          // getStatusForPolicy() exists and doesn't blow up.
          return mediaKeys.getStatusForPolicy({minHdcpVersion: hdcpVersion});
        });
  }

  promise_test(
      () => checkStatusForMinHdcpVersionPolicy(''),
      testnamePrefix(qualifier, config.keysystem) +
          ' support for empty HDCP version.');

  promise_test(
      () => checkStatusForMinHdcpVersionPolicy('1.0'),
      testnamePrefix(qualifier, config.keysystem) + ' support for HDCP 1.0.');
}
