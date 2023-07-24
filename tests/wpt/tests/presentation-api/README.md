# Presentation API Tests

This test suite is currently tracking the [Editor's Draft][editor-draft] of the Presentation API. The Presentation API describes the [conformance criteria for two classes of user agents][conformance-classes] ([controlling user agent][dfn-controlling-user-agent] and [receiving user agent][dfn-receiving-user-agent]). Each of the two subfolders [controlling-ua](./controlling-ua) and [receiving-ua](./receiving-ua) contains the Presentation API tests for each class of user agents.

## IDL Tests

The [controlling-ua](./controlling-ua) and [receiving-ua](./receiving-ua) subfolders contain files `idlharness.https.html` and `idlharness-manual.https.html` that define IDL tests of the Presentation API for controlling and receiving user agents, respectively. The WebIDL of the Presentation API spec is extracted from the [Editor's Draft][editor-draft] by running the following JavaScript code in the Dev. console of the Browser.

```javascript
(function(){
  var s = "";
  [].forEach.call(document.getElementsByClassName("idl"), function(idl) {
    if (!idl.classList.contains("extract"))
      s += idl.textContent + "\n\n";
  });
  document.body.innerHTML = '<pre></pre>';
  document.body.firstChild.textContent = s;
  })();
```

## Receiving User Agent Tests

The [receiving-ua](./receiving-ua) subfolder contains receiving user agent tests to be initiated by _a controlling user agent_. When the controlling user agent starts the test, it will ask a user to click a button and choose a presentation display. Once the presentation display is selected, the controlling user agent will request the receiving user agent to load and run the corresponding test placed in the [receiving-ua/support](./receiving-ua/support) subfolder. When the test ends, all results will appear on the controlling user agent's window.

[editor-draft]: http://w3c.github.io/presentation-api/
[conformance-classes]: http://w3c.github.io/presentation-api/#conformance-classes
[dfn-controlling-user-agent]: http://w3c.github.io/presentation-api/#dfn-controlling-user-agent
[dfn-receiving-user-agent]: http://w3c.github.io/presentation-api/#dfn-receiving-user-agent