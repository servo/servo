const RESOURCES_DIR = "/html/semantics/links/downloading-resources/resources/";

function testReferrerHeader(expectedReferrer) {
  let id = self.token();
  let testUrl = RESOURCES_DIR + "inspect-header.py?header=referer&cmd=put&id=" + id;

  promise_test(function(test) {
    const anchor = document.getElementById("a");
    anchor.setAttribute("ping", testUrl);
    anchor.click();
    return pollResult(id) .then(result => {
      assert_equals(result, expectedReferrer, "Correct referrer header result");
    });
  }, "Test referer header " + RESOURCES_DIR);
}

// Sending a ping is an asynchronous and non-blocking request to a web server.
// We may have to create a poll loop to get result from server
function pollResult(id) {
  let checkUrl = RESOURCES_DIR + "inspect-header.py?header=referer&cmd=get&id=" + id;

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
