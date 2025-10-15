setup({ allow_uncaught_exception: true });

test(t => {
  document.body.append(document.createElement("iframe"), document.createElement("iframe"));
  t.add_cleanup(() => document.querySelectorAll("iframe").forEach(iframe => iframe.remove()));

  const frame0Document = frames[0].document.documentElement;
  const frame1Body = frames[1].document.body;

  frame1Body.setAttribute("onclick", "void(0)");
  frame0Document.appendChild(frame1Body);
  const get = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "onclick").get;
  const f = get.call(frame1Body);

  assert_equals(f.constructor, frames[0].Function, "The function must be created in the element's document's global");
}, "Event handler IDL attributes must return a function from the element's document's realm");

test(t => {
  document.body.append(document.createElement("iframe"), document.createElement("iframe"));
  t.add_cleanup(() => document.querySelectorAll("iframe").forEach(iframe => iframe.remove()));

  const log = [];
  window.addEventListener("error", t.step_func(e => {
    log.push("error event in top / error object realm = " + getErrorRealm(e));
  }, { signal: t.get_signal() }));
  frames[0].addEventListener("error", t.step_func(e => {
    log.push("error event in frames[0] / error object realm = " + getErrorRealm(e));
  }, { signal: t.get_signal() }));
  frames[1].addEventListener("error", t.step_func(e => {
    log.push("error event in frames[1] / error object realm = " + getErrorRealm(e));
  }, { signal: t.get_signal() }));

  const frame0Document = frames[0].document.documentElement;
  const frame1Body = frames[1].document.body;

  frame1Body.setAttribute("onmousedown", "1 *-* 'syntax error'");
  frame0Document.appendChild(frame1Body);

  assert_array_equals(log, [], "No error events must be fired before calling the getter");

  const get = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "onmousedown").get;
  const f = get.call(frame1Body);

  assert_array_equals(log, ["error event in frames[0] / error object realm = frames[0]"]);
  assert_equals(f, null, "The returned value must be null");
}, "Event handler IDL attribute compilation errors must be fired on the element's document's global");

async_test(t => {
  const frame0 = document.createElement("iframe");
  const frame1 = document.createElement("iframe");
  document.body.append(frame0, frame1);
  t.add_cleanup(() => {
    frame0.remove();
    frame1.remove();
  });

  const frame0Document = frame0.contentDocument.documentElement;
  const frame1Img = frame1.contentDocument.createElement('img');
  frame1Img.setAttribute("onload", "window.imageLoaded=true");
  frame0Document.appendChild(frame1Img);

  frame1Img.addEventListener("load", t.step_func_done(() => {
    assert_true(frame0.contentWindow.imageLoaded, "The window object should point to the new document's global");
  }));

  frame1Img.setAttribute("src", "/images/green.png");
}, "Event handler IDL attributes must point window to the element's document's global");

async_test(t => {
  const frame0 = document.createElement("iframe");
  const frame1 = document.createElement("iframe");
  document.body.append(frame0, frame1);
  t.add_cleanup(() => {
    frame0.remove();
  });

  frame1.onload = () => {
    const frame0Document = frame0.contentDocument.documentElement;
    const frame1Img = frame1.contentDocument.querySelector('img');
    frame1Img.addEventListener("load", t.step_func(() => {
      if (frame1Img.getAttribute('src') === '/images/green.png') {
        assert_true(frame0.contentWindow.imageLoaded, "The window object should point to the new document's global");
        t.done();
      }
    }));
    frame0Document.appendChild(frame1Img);
    frame1.remove();
    frame1Img.setAttribute("src", "/images/green.png");
  };

  frame1.srcdoc = "<img src='/images/red.png' onload='window.imageLoaded=true'>";
}, "Event handler IDL attributes must point window to the new element's document's global after old frame is removed");

function getErrorRealm(event) {
  const { error } = event;

  if (error instanceof SyntaxError) {
    return "top";
  } if (error instanceof frames[0].SyntaxError) {
    return "frames[0]";
  } if (error instanceof frames[1].SyntaxError) {
    return "frames[1]";
  }
  return "unknown";
}
