function StreamDownloadFinishDelay() {
    return 1000;
}

function DownloadVerifyDelay() {
    return 1000;
}

function VerifyDownload(test_obj, token, timeout, expect_download) {
    var verify_token = test_obj.step_func(function () {
        var xhr = new XMLHttpRequest();
        xhr.open('GET', 'support/download_stash.py?verify-token&token=' + token);
        xhr.onload = test_obj.step_func(function(e) {
            if (expect_download) {
              if (xhr.response != "TOKEN_SET") {
                // Always retry, and rely on the test timeout to conclude that
                // download didn't happen and to fail the test.
                test_obj.step_timeout(verify_token, DownloadVerifyDelay());
                return;
              }
            } else {
              assert_equals(xhr.response, "TOKEN_NOT_SET", "Expect no download to happen, but got one.");
            }
            test_obj.done();
        });
        xhr.send();
    });
    test_obj.step_timeout(verify_token, timeout);
}

function AssertDownloadSuccess(test_obj, token, timeout) {
    VerifyDownload(test_obj, token, timeout, true);
}

function AssertDownloadFailure(test_obj, token, timeout) {
    VerifyDownload(test_obj, token, timeout, false);
}
