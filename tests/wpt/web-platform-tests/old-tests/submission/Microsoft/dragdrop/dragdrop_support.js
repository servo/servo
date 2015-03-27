function AddEventListenersForElement(evt, callback, capture, element)
{
    element.addEventListener(evt, callback, capture);
}

function LogTestResult(result)
{
    document.getElementById("test_result").firstChild.data = result;
}
