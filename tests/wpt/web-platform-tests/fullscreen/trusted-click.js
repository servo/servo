// Invokes callback from a trusted click event, to satisfy
// https://html.spec.whatwg.org/#triggered-by-user-activation
function trusted_click(test, callback, container)
{
    var document = container.ownerDocument;
    var button = document.createElement("button");
    button.textContent = "click to continue test";
    button.style.display = "block";
    button.style.fontSize = "20px";
    button.style.padding = "10px";
    button.onclick = test.step_func(function()
    {
        callback();
        container.removeChild(button);
    });
    container.appendChild(button);
}

// Invokes element.requestFullscreen() from a trusted click.
function trusted_request(test, element, container)
{
    trusted_click(test, () => element.requestFullscreen(), container || element.parentNode);
}
