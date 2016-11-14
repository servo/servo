(function (window) {
  // Cast ID of the main custom receiver application linked with the test suite
  // That application ID, maintained by W3C team, points at:
  // https://[W3C test server]/presentation-api/controlling-ua/support/presentation.html
  //
  // NB: this mechanism should be improved later on as tests should not depend
  // on something that directly or indirectly maps to a resource on the W3C test
  // server.
  var castAppId = '915D2A2C';
  var castUrl = 'https://google.com/cast#__castAppId__=' + castAppId;

  window.presentationUrls = [
    'support/presentation.html',
    castUrl
  ];
})(window);