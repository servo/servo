// META: script=resources/utils.js

async_test(t => {
    assert_implements(window.PerformanceLongTaskTiming, 'Longtasks are not supported.');
    new PerformanceObserver(t.step_func((entryList, obs) => {
        const observer = new PerformanceObserver(t.step_func_done(list => {
            let longtaskObserved = false;
            list.getEntries().forEach(entry => {
                if (entry.entryType === 'mark')
                    return;
                checkLongTaskEntry(entry);
                longtaskObserved = true;
            });
            assert_true(longtaskObserved, 'Did not observe buffered longtask.');
        }));
        observer.observe({type: 'longtask', buffered: true});
        // Observer mark so that we can flush the observer callback.
        observer.observe({type: 'mark'});
        performance.mark('m');
        // Disconnect the embedding observer so this callback only runs once.
        obs.disconnect();
    })).observe({entryTypes: ['longtask']});
    // Create a long task.
    const begin = window.performance.now();
    while (window.performance.now() < begin + 60);
}, 'PerformanceObserver with buffered flag can see previous longtask entries.');
