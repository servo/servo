function verifyPreloadAndRTSupport()
{
    var link = window.document.createElement("link");
    assert_true(link.relList && link.relList.supports("preload"), "Preload not supported");
    assert_true(!!window.PerformanceResourceTiming, "ResourceTiming not supported");
}

function getAbsoluteURL(url)
{
    return new URL(url, location.href).href;
}

function verifyNumberOfResourceTimingEntries(url, number)
{
    assert_equals(numberOfResourceTimingEntries(url), number, url);
}

function numberOfResourceTimingEntries(url)
{
    return performance.getEntriesByName(getAbsoluteURL(url)).length;
}

// Verifies that the resource is loaded, but not downloaded from network
// more than once. This can be used to verify that a preloaded resource is
// not downloaded again when used.
function verifyLoadedAndNoDoubleDownload(url) {
    var entries = performance.getEntriesByName(getAbsoluteURL(url));
    // UA may create separate RT entries for preload and normal load,
    // so we just check (entries.length > 0).
    assert_greater_than(entries.length, 0, url + ' should be loaded');

    var numDownloads = 0;
    entries.forEach(entry => {
        // transferSize is zero if the resource is loaded from cache.
        if (entry.transferSize > 0) {
            numDownloads++;
        }
    });
    // numDownloads can be zero if the resource was already cached before running
    // the test (for example, when the test is running repeatedly without
    // clearing cache between runs).
    assert_less_than_equal(
        numDownloads, 1,
        url + ' should be downloaded from network at most once');
}
