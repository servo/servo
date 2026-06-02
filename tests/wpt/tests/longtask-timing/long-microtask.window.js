// META: script=resources/utils.js

async_test(function (t) {
    assert_implements(window.PerformanceLongTaskTiming, 'Longtasks are not supported.');
    new PerformanceObserver(
        t.step_func_done(entryList => {
            const entries = entryList.getEntries();
            assert_equals(entries.length, 1,
                'Exactly one entry is expected.');
            const longtask = entries[0];
            checkLongTaskEntry(longtask);
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
