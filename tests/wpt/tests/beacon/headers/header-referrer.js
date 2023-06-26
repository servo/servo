var RESOURCES_DIR = "/beacon/resources/";

var referrerOrigin = self.location.origin + '/';
var referrerUrl = self.location.href;

function testReferrerHeader(testBase, expectedReferrer, mayBeBlockedAsMixedContent = false) {
  var id = self.token();
  var testUrl = testBase + "inspect-header.py?header=referer&cmd=put&id=" + id;

  promise_test(function(test) {
    const sentBeacon = navigator.sendBeacon(testUrl);
    if (mayBeBlockedAsMixedContent && !sentBeacon)
      return Promise.resolve();
    assert_true(sentBeacon, "SendBeacon Succeeded");
    return pollResult(expectedReferrer, id) .then(result => {
      assert_equals(result, expectedReferrer, "Correct referrer header result");
    });
  }, "Test referer header " + testBase);
}

// SendBeacon is an asynchronous and non-blocking request to a web server.
// We may have to create a poll loop to get result from server
function pollResult(expectedReferrer, id) {
  var checkUrl = RESOURCES_DIR + "inspect-header.py?header=referer&cmd=get&id=" + id;

  return new Promise(resolve => {
    function checkResult() {
      fetch(checkUrl).then(
        function(response) {
          assert_equals(response.status, 200, "Inspect header response's status is 200");
          let result = response.headers.get("x-request-referer");

          if (result != undefined) {
            resolve(result);
          } else {
            step_timeout(checkResult.bind(this), 100);
          }
        });
    }

    checkResult();
  });

}
