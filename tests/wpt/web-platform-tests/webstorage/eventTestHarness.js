iframe = document.createElement("IFRAME");
iframe.src = "about:blank";
document.body.appendChild(iframe);
iframe.contentWindow.document.body.innerText = "Nothing to see here.";

storageEventList = new Array();
iframe.contentWindow.onstorage = function (e) {
    window.parent.storageEventList.push(e);
}

function runAfterNStorageEvents(callback, expectedNumEvents)
{
    countStorageEvents(callback, expectedNumEvents, 0)
}

function countStorageEvents(callback, expectedNumEvents, times)
{
    function onTimeout()
    {
        var currentCount = storageEventList.length;
        if (currentCount == expectedNumEvents) {
            callback();
        } else if (currentCount > expectedNumEvents) {
            msg = "got at least " + currentCount + ", expected only " + expectedNumEvents + " events";
            callback(msg);
        } else if (times > 50) {
            msg = "Timeout: only got " + currentCount + ", expected " + expectedNumEvents + " events";
            callback(msg);
        } else {
            countStorageEvents(callback, expectedNumEvents, times+1);
        }
    }
    setTimeout(onTimeout, 20);
}

function testStorages(testCallback)
{
    // When we're done testing LocalStorage, this is run.
    function allDone()
    {
        localStorage.clear();
        sessionStorage.clear();
    }

    // When we're done testing with SessionStorage, this is run.
    function runLocalStorage()
    {
        testCallback("localStorage", allDone);
    }

    // First run the test with SessionStorage.
    testCallback("sessionStorage", runLocalStorage);
}
