async_test(function (t) {
    if (typeof PerformanceLongTaskTiming === 'undefined') {
        assert_unreached("Longtasks are not supported.");
        t.done();
    }
    new PerformanceObserver(
        t.step_func_done(entryList => {
            const entries = entryList.getEntries();
            assert_equals(entries.length, 1,
                'Exactly one entry is expected.');
            const longtask = entries[0];
            assert_equals(longtask.entryType, 'longtask');
            assert_equals(longtask.name, 'self');
            assert_greater_than(longtask.duration, 50);
            assert_greater_than_equal(longtask.startTime, 0);
            const currentTime = performance.now();
            assert_less_than_equal(longtask.startTime, currentTime);
            t.done();
        })
    ).observe({entryTypes: ['longtask']});

    window.onload = () => {
        /* Generate a slow microtask */
        Promise.resolve().then(() => {
            const begin = window.performance.now();
            while (window.performance.now() < begin + 60);
        });
    };
}, 'A short task followed by a long microtask is observable.');
