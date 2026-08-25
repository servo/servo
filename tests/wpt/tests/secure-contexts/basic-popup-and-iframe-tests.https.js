
/**
 * This test checks the Secure Context state of documents for various
 * permutations of document URI types and loading methods.
 *
 * The hierarchy that is tested is:
 *
 *   creator-doc > createe-doc
 *
 * The creator-doc is one of:
 *
 *   http:
 *   https:
 *
 * The createe-doc is loaded as either a:
 *
 *   popup
 *   iframe
 *   sandboxed-iframe
 *
 * into which we load and test:
 *
 *   http:
 *   https:
 *   blob:
 *   javascript:
 *   about:blank
 *   initial about:blank
 *   srcdoc
 *
 * TODO once web-platform-tests supports it:
 *   - test http://localhost
 *   - test file:
 */


setup({explicit_done:true});


const host_and_dirname = location.host +
                         location.pathname.substr(0, location.pathname.lastIndexOf("/") + 1);


// Flags to indicate where document types should be loaded for testing:
const eLoadInPopup             = (1 << 0);
const eLoadInUnsandboxedIframe = (1 << 1);
const eLoadInSandboxedIframe   = (1 << 2);
const eLoadInEverything        = eLoadInPopup | eLoadInUnsandboxedIframe | eLoadInSandboxedIframe;

// Flags indicating if a document type is expected to be a Secure Context:
const eSecureNo              = 1;
const eSecureIfNewWindow     = 2;
const eSecureIfCreatorSecure = 3;

// Flags indicating how the result of a test is obtained:
const eResultFromPostMessage       = 1;
const eResultFromExaminationOnLoad = 2;
const eResultFromExaminationSync   = 3;


const loadTypes = [
  new LoadType("an http: URI",
               eLoadInEverything,
               http_dir + "postMessage-helper.html",
               eSecureNo,
               eResultFromPostMessage),
  new LoadType("an https: URI",
               eLoadInEverything,
               https_dir + "postMessage-helper.https.html",
               eSecureIfNewWindow,
               eResultFromPostMessage),
  new LoadType("a blob: URI",
               eLoadInEverything,
               URL.createObjectURL(new Blob(["<script>(opener||parent).postMessage(isSecureContext, '*')</script>"], {type: "text/html"})),
               eSecureIfCreatorSecure,
               eResultFromPostMessage),
  new LoadType("a srcdoc",
               // popup not relevant:
               eLoadInUnsandboxedIframe | eLoadInSandboxedIframe,
               "<script>(opener||parent).postMessage(isSecureContext, '*')</script>",
               eSecureIfNewWindow,
               eResultFromPostMessage),
  new LoadType("a javascript: URI",
               // can't load in sandbox:
               eLoadInUnsandboxedIframe | eLoadInPopup,
               "javascript:(opener||parent).postMessage(isSecureContext, '*')",
               eSecureIfCreatorSecure,
               eResultFromPostMessage),
  new LoadType("about:blank",
               // can't obtain state if sandboxed:
               eLoadInUnsandboxedIframe | eLoadInPopup,
               "about:blank",
               eSecureIfCreatorSecure,
               eResultFromExaminationOnLoad),
  new LoadType("initial about:blank",
               // can't obtain state if sandboxed:
               eLoadInUnsandboxedIframe | eLoadInPopup,
               "about:blank", // we don't wait for this to load, so whatever
               eSecureIfCreatorSecure,
               eResultFromExaminationSync),
  new LoadType("a data: URL",
               // can't load in a top-level browsing context
               eLoadInUnsandboxedIframe | eLoadInSandboxedIframe,
               "data:text/html,<script>parent.postMessage(isSecureContext, '*')</script>",
               eSecureIfCreatorSecure,
               eResultFromPostMessage),
];

const loadTargets = [
  new LoadTarget("an iframe",          eLoadInUnsandboxedIframe),
  new LoadTarget("a sandboxed iframe", eLoadInSandboxedIframe),
  new LoadTarget("a popup",            eLoadInPopup),
];


function LoadType(description, loadInFlags, uri, expectedSecureFlag, resultFrom) {
  this.desc = description;
  this.loadInFlags = loadInFlags;
  this.uri = uri;
  this.expectedSecureFlag = expectedSecureFlag;
  this.resultFrom = resultFrom;
}


function LoadTarget(description, loadInFlag) {
  this.desc = description;
  this.loadInFlag = loadInFlag;
}

LoadTarget.prototype.open = function(loadType) {
  let loadTarget = this;
  this.currentTest.step(function() {
    assert_true((loadTarget.loadInFlag & loadType.loadInFlags) != 0,
                loadType.desc + " cannot be tested in " + loadTarget.desc);
  });
  if (this.loadInFlag == eLoadInUnsandboxedIframe) {
    let iframe = document.createElement("iframe");
    document.body.appendChild(iframe);
    iframe[loadType.desc == "a srcdoc" ? "srcdoc" : "src"] = loadType.uri;
    return iframe;
  }
  if (this.loadInFlag == eLoadInSandboxedIframe) {
    let iframe = document.body.appendChild(document.createElement("iframe"));
    iframe.setAttribute("sandbox", "allow-scripts");
    iframe[loadType.desc == "a srcdoc" ? "srcdoc" : "src"] = loadType.uri;
    return iframe;
  }
  if (this.loadInFlag == eLoadInPopup) {
    return window.open(loadType.uri);
  }
  this.currentTest.step(function() {
    assert_unreached("Unknown load type flag: " + loadInFlags);
  });
  return null;
}

LoadTarget.prototype.close = function(domTarget) {
  if (this.loadInFlag == eLoadInUnsandboxedIframe ||
      this.loadInFlag == eLoadInSandboxedIframe) {
    domTarget.remove();
    return;
  }
  if (this.loadInFlag == eLoadInPopup) {
    domTarget.close();
    return;
  }
  this.currentTest.step(function() {
    assert_unreached("Unknown load type flag: " + loadInFlags);
  });
}

LoadTarget.prototype.load_and_get_result_for = function(loadType) {
  if (!(loadType.loadInFlags & this.loadInFlag)) {
    return Promise.reject("not applicable");
  }
  if (!(this.loadInFlag & eLoadInPopup) &&
      location.protocol == "https:" &&
      loadType.uri.substr(0,5) == "http:") {
    // Mixed content blocker will prevent this load
    return Promise.reject("not applicable");
  }
  this.currentTest = async_test("Test Window.isSecureContext in " + this.desc +
                                " loading " + loadType.desc)
  if (loadType.resultFrom == eResultFromExaminationSync) {
    let domTarget = this.open(loadType);
    let isFrame = domTarget instanceof HTMLIFrameElement;
    let result = !isFrame ?
          domTarget.isSecureContext : domTarget.contentWindow.isSecureContext;
    this.close(domTarget);
    return Promise.resolve({ result: result, isFrame: isFrame});
  }
  let target = this;
  if (loadType.resultFrom == eResultFromExaminationOnLoad) {
    return new Promise(function(resolve, reject) {
      function handleLoad(event) {
        clearTimeout(timer);
        let isFrame = domTarget instanceof HTMLIFrameElement;
        let result = !isFrame ?
              domTarget.isSecureContext : domTarget.contentWindow.isSecureContext;
        domTarget.removeEventListener("load", handleLoad);
        target.close(domTarget);
        resolve({ result: result, isFrame: isFrame});
      }
      let domTarget = target.open(loadType);
      domTarget.addEventListener("load", handleLoad, false);

      // Some browsers don't fire `load` events for `about:blank`. That's weird, but it also
      // isn't what we're testing here.
      let timer = setTimeout(handleLoad, 500);
    });
  }
  if (loadType.resultFrom == eResultFromPostMessage) {
    return new Promise(function(resolve, reject) {
      function handleMessage(event) {
        let isFrame = domTarget instanceof HTMLIFrameElement;
        window.removeEventListener("message", handleMessage);
        target.close(domTarget);
        resolve({ result: event.data, isFrame: isFrame});
      }
      window.addEventListener("message", handleMessage, false);
      let domTarget = target.open(loadType);
   });
  }
  return Promise.reject("unexpected 'result from' type");
}


let current_type_index = -1;
let current_target_index = 0;

function run_next_test() {
  current_type_index++;
  if (current_type_index >= loadTypes.length) {
    current_type_index = 0;
    current_target_index++;
    if (current_target_index >= loadTargets.length) {
      done();
      return; // all test permutations complete
    }
  }
  let loadTarget = loadTargets[current_target_index];
  let loadType = loadTypes[current_type_index];
  loadTarget.load_and_get_result_for(loadType).then(
    function(value) {
      run_next_test_soon();
      loadTarget.currentTest.step(function() {
        // If the new context is always non-secure, the assertion is straightforward.
        if (loadType.expectedSecureFlag == eSecureNo) {
          assert_false(value.result, loadType.desc + " in " + loadTarget.desc + " should not create a Secure Context");
        // If the new context is always secure if opened in a new window, and it's
        // been opened in a new window, the assertion is likewise straightforward.
        } else if (loadType.expectedSecureFlag == eSecureIfNewWindow && !value.isFrame) {
          assert_true(value.result, loadType.desc + " in " + loadTarget.desc + " should create a secure context regardless of its creator's state.");
        // Otherwise, we're either dealing with a context that's secure if and only
        // if its creator context (e.g. this window) is secure.
        } else if ((loadType.expectedSecureFlag == eSecureIfNewWindow && value.isFrame) ||
                   (loadType.expectedSecureFlag == eSecureIfCreatorSecure)) {
          if (!window.isSecureContext) {
            assert_false(value.result, loadType.desc + " in " + loadTarget.desc + " should not create a Secure Context when its creator is not a Secure Context.");
          } else {
            assert_true(value.result, loadType.desc + " in " + loadTarget.desc + " should create a Secure Context when its creator is a Secure Context");
          }
        } else {
          assert_unreached(loadType.desc + " - unknown expected secure flag: " + expectedSecureFlag);
        }
        loadTarget.currentTest.done();
      });
    },
    function(failReason) {
      run_next_test_soon();
      if (failReason == "not applicable") {
        return;
      }
      loadTarget.currentTest.step(function() {
        assert_unreached(loadType.desc + " - got unexpected rejected promise");
      });
    }
  );
}

function run_next_test_soon() {
  setTimeout(run_next_test, 0);
}

function begin() {
  test(function() {
    if (location.protocol == "http:") {
      assert_false(isSecureContext,
                   "http: creator should not be a Secure Context");
    } else if (location.protocol == "https:") {
      assert_true(isSecureContext,
                  "https: creator should be a Secure Context");
    } else {
      assert_unreached("Unknown location.protocol");
    }
  });
  run_next_test();
}

