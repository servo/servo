async_test(t => {
    if (!window.PerformanceLongTaskTiming) {
        assert_unreached("Longtasks are not supported.");
    }
    new PerformanceObserver(t.step_func(entryList => {
        const observer = new PerformanceObserver(t.step_func_done(list => {
            list.getEntries().forEach(entry => {
                assert_not_equals(entry.entryType, 'longtask');
            });
        }));
        observer.observe({type: 'longtask', buffered: true});
        observer.observe({type: 'mark'});
        // Create a mark to ensure the |observer|'s callback is dispatched.
        performance.mark('a');
    })).observe({entryTypes: ['longtask']});
    // Create a long task.
    const begin = window.performance.now();
    while (window.performance.now() < begin + 60);
}, 'PerformanceObserver with buffered flag cannot see previous longtask entries.');
