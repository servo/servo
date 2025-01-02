'use strict';

// https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer
const REFERRER_ORIGIN = self.location.origin + '/';
const REFERRER_URL = self.location.href;

function testReferrerHeader(id, host, expectedReferrer) {
  const url = `${
      host}/beacon/resources/inspect-header.py?header=referer&cmd=put&id=${id}`;

  promise_test(t => {
    fetchLater(url, {activateAfter: 0});
    return pollResult(expectedReferrer, id).then(result => {
      assert_equals(result, expectedReferrer, 'Correct referrer header result');
    });
  }, `Test referer header ${host}`);
}

function pollResult(expectedReferrer, id) {
  const checkUrl =
      `/beacon/resources/inspect-header.py?header=referer&cmd=get&id=${id}`;

  return new Promise(resolve => {
    function checkResult() {
      fetch(checkUrl).then(response => {
        assert_equals(
            response.status, 200, 'Inspect header response\'s status is 200');
        let result = response.headers.get('x-request-referer');

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
