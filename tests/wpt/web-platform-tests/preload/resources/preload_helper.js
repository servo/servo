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

function verifyNumberOfDownloads(url, number)
{
    var numDownloads = 0;
    performance.getEntriesByName(getAbsoluteURL(url)).forEach(entry => {
        if (entry.transferSize > 0) {
            numDownloads++;
        }
    });
    assert_equals(numDownloads, number, url);
}
