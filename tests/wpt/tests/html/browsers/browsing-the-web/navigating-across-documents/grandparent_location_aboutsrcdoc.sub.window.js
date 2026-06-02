// META: script=./resources/testharness.js
// META: script=./resources/testharnessreport.js

function run_test(cross_origin, same_doc, sandbox, name) {
  promise_test(async t => {
    const child_frame = document.createElement('iframe');

    const child_relative_path = './resources/child_with_srcdoc_subframe.window.html';
    if (cross_origin) {
      const new_origin = new URL('http://{{hosts[][www]}}:{{ports[http][1]}}');
      const child_url_same_site = new URL(child_relative_path, location.href);
      child_frame.src = new_origin.origin + child_url_same_site.pathname;
    } else {
      child_frame.src = child_relative_path;
    }
    const iframe_load = new Promise(resolve => {
      child_frame.onload = resolve;
    });
    document.body.appendChild(child_frame);
    await iframe_load;

    let load_count = 0;
    const test_finished = new Promise(resolve => {
      window.onmessage = (e) => {
        load_count++;
        if (load_count == 1) {
          assert_equals(e.data, "about:srcdoc");
          // Allow the main frame to try and set the grand child's location.
          if (same_doc) {
            if (!sandbox) {
              // If `sandbox` is set, the child will self-navigate, otherwise
              // the main frame initiates the same-document navigation.
              frames[0][0].location = "about:srcdoc#the_anchor";
            }
          } else {
            frames[0][0].location = "about:srcdoc";
          }
        } else if (load_count == 2) {
          if (same_doc) {
            // The result for same_doc is the same whether cross_origin is set
            // or not.
            assert_equals(e.data, "about:srcdoc#the_anchor");
          } else if (cross_origin) {
            const child_url = new URL(child_frame.src);
            const expected_data = "SecurityError: " +
                    "Failed to read a named property 'href' from 'Location': " +
                    "Blocked a frame with origin \"" + child_url.origin +
                    "\" from accessing a cross-origin frame."
            assert_equals(String(e.data), expected_data);
          } else {
            assert_equals(e.data, "about:srcdoc");
          }
          resolve();
        }
      }
    });
    let cmd_str = "load grandchild";
    if (sandbox) {
      cmd_str += " sandbox";
    }
    child_frame.contentWindow.postMessage(cmd_str, "*");
    await test_finished;

    t.add_cleanup(() => child_frame.remove())
  }, name);
}

onload = () => {
  // A cross-origin frame cannot set about:srcdoc but can do same-doc navigations.
  run_test(cross_origin = true, same_doc = false, sandbox = false, name =
      "cross-origin grandparent sets location to about:srcdoc");
  run_test(cross_origin = true, same_doc = true, sandbox = false, name =
      "cross-origin grandparent sets location in same-doc navigation");

  // A same-origin frame can set about:srcdoc and also do same-doc navigations.
  run_test(cross_origin = false, same_doc = false, sandbox = false, name =
      "same-origin grandparent sets location to about:srcdoc");
  run_test(cross_origin = false, same_doc = true, sandbox = false, name =
      "same-origin grandparent sets location in same-doc navigation");

  // For the sandboxed srcdoc cases, the srcdoc will be cross-origin to
  // everything but itself, but it should be able to navigate itself same-
  // document.
  run_test(cross_origin = false, same_doc = true, sandbox = true, name =
      "same-origin grandparent with sandboxed srcdoc grandchild that self navigates");
  run_test(cross_origin = true, same_doc = true, sandbox = true, name =
      "cross-origin grandparent with sandboxed srcdoc grandchild that self navigate");
};
