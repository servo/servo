/**
 * This is the guts of the load/error event tests for <link rel="stylesheet">.
 *
 * We have a list of tests each of which is an object containing: href value,
 * expected load success boolean, test description.  Href values are set up in
 * such a way that we guarantee that all stylesheet URLs are unique.  This
 * avoids issues around caching of sheets based on URL.
 */

// Our URLs are random, so we don't use them in error messages by
// default, but enable doing it if someone wants to debug things.
const DEBUG_URLS = false;

var isHttps = location.protocol == "https:";

var tests = [
  // Basic tests
  {
    href: existingSheet(),
    success: true,
    description: "Basic load of stylesheet",
  },
  {
    href: nonexistentSheet(),
    success: false,
    description: "Attempted load of nonexistent stylesheet",
  },
  {
    href: `data:text/css,@import url("${existingSheet()}")`,
    success: true,
    description: "Import of stylesheet",
  },
  {
    href: `data:text/css,@import url("${nonexistentSheet()}")`,
    success: false,
    description: "Import of nonexistent stylesheet",
  },
  {
    href: `data:text/css,@import url("data:text/css,@import url('${existingSheet()}')")`,
    success: true,
    description: "Import of import of stylesheet",
  },
  {
    href: `data:text/css,@import url("data:text/css,@import url('${nonexistentSheet()}')")`,
    success: false,
    description: "Import of import of nonexistent stylesheet",
  },

  // Non-CSS-response tests.
  {
    href: makeUnique(""),
    success: false,
    description: "Load of non-CSS stylesheet",
  },
  {
    href: `data:text/css,@import url("${makeUnique("")}")`,
    success: false,
    description: "Import of non-CSS stylesheet",
  },
  {
    href: `data:text/css,@import url("data:text/css,@import url('${makeUnique("")}')")`,
    success: false,
    description: "Import of import of non-CSS stylesheet",
  },

  // http:// tests, to test what happens with mixed content blocking.
  {
    href: httpSheet(),
    success: !isHttps,
    description: "Load of http:// stylesheet",
  },
  {
    href: `data:text/css,@import url("${httpSheet()}")`,
    success: !isHttps,
    description: "Import of http:// stylesheet",
  },
  {
    href: `data:text/css,@import url("data:text/css,@import url('${httpSheet()}')")`,
    success: !isHttps,
    description: "Import of import of http:// stylesheet",
  },

  // https:// tests just as a control
  {
    href: httpsSheet(),
    success: true,
    description: "Load of https:// stylesheet",
  },
  {
    href: `data:text/css,@import url("${httpsSheet()}")`,
    success: true,
    description: "Import of https:// stylesheet",
  },
  {
    href: `data:text/css,@import url("data:text/css,@import url('${httpsSheet()}')")`,
    success: true,
    description: "Import of import of https:// stylesheet",
  },

  // Tests with multiple imports some of which are slow and some are fast.
  {
    href: `data:text/css,@import url("${slowResponse(existingSheet())}"); @import url("${nonexistentSheet()}");`,
    success: false,
    description: "Slow successful import, fast failing import",
  },
  {
    href: `data:text/css,@import url("${existingSheet()}"); @import url("${slowResponse(nonexistentSheet())}");`,
    success: false,
    description: "Fast successful import, slow failing import",
  }
];

// Note: Here we really do need to use "let" at least for the href,
// because we lazily evaluate it in the unreached cases.
for (var test of tests) {
  let {href, success, description} = test;
  var t = async_test(description);
  var link = document.createElement("link");
  link.rel = "stylesheet";
  hrefString = DEBUG_URLS ? `: ${href}` : "";
  if (success) {
    link.onload = t.step_func_done(() => {});
    link.onerror = t.step_func_done(() => assert_unreached(`error fired when load expected${hrefString}`) );
  } else {
    link.onerror = t.step_func_done(() => {});
    link.onload = t.step_func_done(() => assert_unreached(`load fired when error expected${hrefString}`) );
  }
  link.href = href;
  document.head.appendChild(link);
}

/* Utility function */
function makeUnique(url) {
  // Make sure we copy here, even if the thing coming in is a URL, so we don't
  // mutate our caller's data.
  url = new URL(url, location.href);
  // We want to generate a unique URI to avoid the various caches browsers have
  // for stylesheets.  We don't want to just use a counter, because that would
  // not be robust to the test being reloaded or othewise run multiple times
  // without a browser restart.  We don't want to use timstamps, because those
  // are not likely to be unique across calls to this function, especially given
  // the degraded timer resolution browsers have due to Spectre.
  //
  // So just fall back on Math.random() and assume it can't duplicate values.
  url.searchParams.append("r", Math.random());
  return url;
}

function existingSheet() {
  return makeUnique("resources/good.css");
}

/**
 * Function the add values to the "pipe" search param.  See
 * http://wptserve.readthedocs.io/en/latest/pipes.html for why one would do
 * this.  Because this param uses a weird '|'-separated syntax instead of just
 * using multiple params with the same name, we need some manual code to munge
 * the value properly.
 */
function addPipe(url, pipeVal) {
  url = new URL(url, location.href);
  var params = url.searchParams;
  var oldVal = params.get("pipe");
  if (oldVal) {
    params.set("pipe", oldVal + "|" + pipeVal);
  } else {
    params.set("pipe", pipeVal);
  }
  return url;
}

function nonexistentSheet() {
  return addPipe(existingSheet(), "status(404)");
}

function httpSheet() {
  var url = existingSheet();
  url.protocol = "http";
  url.port = {{ports[http][0]}};
  return url;
}

function httpsSheet() {
  var url = existingSheet();
  url.protocol = "https";
  url.port = {{ports[https][0]}};
  return url;
}

function slowResponse(url) {
  return addPipe(url, "trickle(d1)");
}
