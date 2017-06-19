setup({output:%(output)d});

add_completion_callback(function() {
    add_completion_callback(function (tests, status) {
        var subtest_results = tests.map(function(x) {
            return [x.name, x.status, x.message, x.stack]
        });
        var id = location.pathname + location.search + location.hash;
        var results = JSON.stringify([id,
                                      status.status,
                                      status.message,
                                      status.stack,
                                      subtest_results]);
        (function done() {
            if (window.__wd_results_callback__) {
                clearTimeout(__wd_results_timer__);
                __wd_results_callback__(results)
            } else {
                setTimeout(done, 20);
            }
        })()
    })
});
