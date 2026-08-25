function get_test_results(id) {
    async_test(function(test) {
        test.step_timeout(loop, 100);
        function loop() {
            var xhr = new XMLHttpRequest();
            xhr.open('GET', 'stash.py?id=' + id);
            xhr.onload = test.step_func(function() {
                assert_equals(xhr.status, 200);
                if (xhr.responseText) {
                    assert_equals(xhr.responseText, "OK");
                    test.done();
                } else {
                    test.step_timeout(loop, 100);
                }
            });
            xhr.send();
        }
    });
}

function send_test_results(results) {
    var ok = true;
    for (result in results) { ok = ok && results[result]; }
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'stash.py?id=' + results.id);
    xhr.send(ok ? "OK" : "FAIL: " + JSON.stringify(results));
}
