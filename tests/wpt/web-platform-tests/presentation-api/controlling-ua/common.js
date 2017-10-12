(function (window) {
  // Cast ID of the main custom receiver application linked with the test suite
  // That application ID, maintained by W3C team, points at:
  // https://[W3C test server]/presentation-api/controlling-ua/support/presentation.html
  //
  // NB: this mechanism should be improved later on as tests should not depend
  // on something that directly or indirectly maps to a resource on the W3C test
  // server.
  var castAppId = '915D2A2C';
  var castUrl = 'cast:' + castAppId;

  window.presentationUrls = [
    'support/presentation.html',
    castUrl
  ];

  // Both a controlling side and a receiving one must share the same Stash ID to
  // transmit data from one to the other. On the other hand, due to polling mechanism
  // which cleans up a stash, stashes in both controller-to-receiver direction
  // and one for receiver-to-controller are necessary.
  window.stashIds = {
    toController: '9bf08fea-a71a-42f9-b3c4-fa19499e4d12',
    toReceiver: 'f1fdfd10-b606-4748-a644-0a8e9df3bdd6'
  }
})(window);
