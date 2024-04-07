// NOTE: this file needs to be split up rather than expanded. See ../location.sub.html for some
// extracted tests. Tracked by https://github.com/web-platform-tests/wpt/issues/4934.

/*
* help:
*   https://html.spec.whatwg.org/multipage/#the-link-element
*   https://html.spec.whatwg.org/multipage/#styling
*   https://html.spec.whatwg.org/multipage/#prepare-a-script
*   https://html.spec.whatwg.org/multipage/#concept-media-load-algorithm
*   https://html.spec.whatwg.org/multipage/#track-url
*   https://html.spec.whatwg.org/multipage/#concept-form-submit
*   https://html.spec.whatwg.org/multipage/#set-the-frozen-base-url
*   https://dom.spec.whatwg.org/#dom-node-baseuri
*   https://html.spec.whatwg.org/multipage/#the-a-element
*   https://html.spec.whatwg.org/multipage/#dom-worker
*   https://html.spec.whatwg.org/multipage/#dom-sharedworker
*   https://html.spec.whatwg.org/multipage/#dom-eventsource
*   https://html.spec.whatwg.org/multipage/#dom-xmldocument-load
*   https://html.spec.whatwg.org/multipage/#dom-open
*   http://url.spec.whatwg.org/#dom-url-search
*   https://www.w3.org/Bugs/Public/show_bug.cgi?id=24148
*   https://xhr.spec.whatwg.org/#the-open()-method
*   https://html.spec.whatwg.org/multipage/#set-up-a-worker-script-settings-object
*   https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-importscripts
*   https://html.spec.whatwg.org/multipage/#parse-a-websocket-url's-components
*   https://html.spec.whatwg.org/multipage/#dom-websocket-url
*   https://www.w3.org/Bugs/Public/show_bug.cgi?id=23968
*   http://dev.w3.org/csswg/cssom/#requirements-on-user-agents-implementing-the-xml-stylesheet-processing-instruction
*   http://url.spec.whatwg.org/#dom-url
*/
setup({explicit_done:true});
onload = function() {
  var encoding = '{{GET[encoding]}}';
  var input_url = 'resources/resource.py?q=\u00E5&encoding=' + encoding + '&type=';
  ('html css js worker sharedworker worker_importScripts sharedworker_importScripts worker_worker worker_sharedworker sharedworker_worker '+
   'sharedworker_sharedworker eventstream png svg xmlstylesheet_css video webvtt').split(' ').forEach(function(str) {
    window['input_url_'+str] = input_url + str;
  });
  var blank = 'resources/blank.py?encoding=' + encoding;
  var stash_put = 'resources/stash.py?q=\u00E5&action=put&id=';
  var stash_take = 'resources/stash.py?action=take&id=';
  var expected_obj = {
    'utf-8':'%C3%A5',
    'utf-16be':'%C3%A5',
    'utf-16le':'%C3%A5',
    'windows-1252':'%E5',
    'windows-1251':'%26%23229%3B'
  };
  var expected_current = expected_obj[encoding];
  var expected_utf8 = expected_obj['utf-8'];

  function msg(expected, got) {
    return 'expected substring '+expected+' got '+got;
  }

  function poll_for_stash(test_obj, uuid, expected) {
    var start = new Date();
    var poll = test_obj.step_func(function () {
      var xhr = new XMLHttpRequest();
      xhr.open('GET', stash_take + uuid);
      xhr.onload = test_obj.step_func(function(e) {
        if (xhr.response == "") {
          if (new Date() - start > 10000) {
            // If we set the status to TIMEOUT here we avoid a race between the
            // page and the test timing out
            test_obj.force_timeout();
          }
          test_obj.step_timeout(poll, 200);
        } else {
          assert_equals(xhr.response, expected);
          test_obj.done();
        }
      });
      xhr.send();
    })
    test_obj.step_timeout(poll, 200);
  }

  // loading html (or actually svg to support <embed>)
  function test_load_nested_browsing_context(tag, attr, spec_url) {
    subsetTestByKey('nested-browsing', async_test, function() {
      var id = 'test_load_nested_browsing_context_'+tag;
      var elm = document.createElement(tag);
      elm.setAttribute(attr, input_url_svg);
      elm.name = id;
      document.body.appendChild(elm);
      this.add_cleanup(function() {
        document.body.removeChild(elm);
      });
      elm.onload = this.step_func_done(function() {
        assert_equals(window[id].document.documentElement.textContent, expected_current);
      });

    }, 'load nested browsing context <'+tag+' '+attr+'>');
  }

  spec_url_load_nested_browsing_context = {
    frame:'https://html.spec.whatwg.org/multipage/#process-the-frame-attributes',
    iframe:'https://html.spec.whatwg.org/multipage/#process-the-iframe-attributes',
    object:'https://html.spec.whatwg.org/multipage/#the-object-element',
    embed:'https://html.spec.whatwg.org/multipage/#the-embed-element-setup-steps'
  };

  'frame src, iframe src, object data, embed src'.split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_load_nested_browsing_context(arr[0], arr[1], spec_url_load_nested_browsing_context[arr[0]]);
  });

  // loading css with <link>
  subsetTestByKey('loading', async_test, function() {
    var elm = document.createElement('link');
    elm.href = input_url_css;
    elm.rel = 'stylesheet';
    document.head.appendChild(elm);
    this.add_cleanup(function() {
      document.head.removeChild(elm);
    });
    elm.onload = this.step_func_done(function() {
      var got = elm.sheet.href;
      assert_true(elm.sheet.href.indexOf(expected_current) > -1, 'sheet.href ' + msg(expected_current, got));
      assert_equals(elm.sheet.cssRules[0].style.content, '"'+expected_current+'"', 'sheet.cssRules[0].style.content');
    });
  }, 'loading css <link>');

  // loading js
  subsetTestByKey('loading-css', async_test, function() {
    var elm = document.createElement('script');
    elm.src = input_url_js + '&var=test_load_js_got';
    document.head.appendChild(elm); // no cleanup
    elm.onload = this.step_func_done(function() {
      assert_equals(window.test_load_js_got, expected_current);
    });
  }, 'loading js <script>');

  // loading image
  function test_load_image(tag, attr, spec_url) {
    subsetTestByKey('loading', async_test, function() {
      var elm = document.createElement(tag);
      if (tag == 'input') {
        elm.type = 'image';
      }
      elm.setAttribute(attr, input_url_png);
      document.body.appendChild(elm);
      this.add_cleanup(function() {
        document.body.removeChild(elm);
      });
      elm.onload = this.step_func_done(function() {
        var got = elm.offsetWidth;
        assert_equals(got, query_to_image_width[expected_current], msg(expected_current, image_width_to_query[got]));
      });
      // <video poster> doesn't notify when the image is loaded so we need to poll :-(
      var interval;
      var check_video_width = function() {
        var width = elm.offsetWidth;
        if (width != 300 && width != 0) {
          clearInterval(interval);
          elm.onload();
        }
      }
      if (tag == 'video') {
        interval = setInterval(check_video_width, 10);
      }
    }, 'loading image <'+tag+' '+attr+'>');
  }

  var query_to_image_width = {
    '%E5':1,
    '%26%23229%3B':1,
    '%C3%A5':2,
    '%3F':16,
    'unknown query':256,
    'default intrinsic width':300
  };

  var image_width_to_query = {};
  (function() {
    for (var x in query_to_image_width) {
      image_width_to_query[query_to_image_width[x]] = x;
    }
  })();

  var spec_url_load_image = {
    img:'https://html.spec.whatwg.org/multipage/#update-the-image-data',
    embed:'https://html.spec.whatwg.org/multipage/#the-embed-element-setup-steps',
    object:'https://html.spec.whatwg.org/multipage/#the-object-element',
    input:'https://html.spec.whatwg.org/multipage/#image-button-state-(type=image)',
    video:'https://html.spec.whatwg.org/multipage/#poster-frame'
  };

  'img src, embed src, object data, input src, video poster'.split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_load_image(arr[0], arr[1], spec_url_load_image[arr[0]]);
  });

  // XXX test <img srcset> or its successor

  // loading video
  function test_load_video(tag, use_source_element) {
    subsetTestByKey('loading', async_test, function() {
      var elm = document.createElement(tag);
      var video_ext = '';
      if (elm.canPlayType('video/webm; codecs="vp9,opus"')) {
        video_ext = 'webm';
      } else if (elm.canPlayType('video/mp4; codecs="avc1.42E01E,mp4a.40.2"')) {
        video_ext = 'mp4';
      }
      assert_not_equals(video_ext, '', 'no supported video format');
      var source;
      if (use_source_element) {
        source = document.createElement('source');
        elm.appendChild(source);
      } else {
        source = elm;
      }
      source.src = input_url_video + '&ext=' + video_ext;
      elm.preload = 'auto';
      elm.load();
      this.add_cleanup(function() {
        elm.removeAttribute('src');
        if (elm.firstChild) {
          elm.removeChild(elm.firstChild);
        }
        elm.load();
      });
      elm.onloadedmetadata = this.step_func_done(function() {
        var got = Math.round(elm.duration);
        assert_equals(got, query_to_video_duration[expected_current], msg(expected_current, video_duration_to_query[got]));
      });
    }, 'loading video <'+tag+'>' + (use_source_element ? '<source>' : ''));
  }

  var query_to_video_duration = {
    '%E5':3,
    '%26%23229%3B':3,
    '%C3%A5':5,
    '%3F':30,
    'unknown query':300,
    'Infinity':Infinity,
    'NaN':NaN
  };

  var video_duration_to_query = {};
  (function() {
    for (var x in query_to_video_duration) {
      video_duration_to_query[query_to_video_duration[x]] = x;
    }
  })();

  'video, audio'.split(', ').forEach(function(str) {
    test_load_video(str);
    test_load_video(str, true);
  });

  // loading webvtt
  subsetTestByKey('loading', async_test, function() {
    var video = document.createElement('video');
    var track = document.createElement('track');
    video.appendChild(track);
    track.src = input_url_webvtt;
    track.track.mode = 'showing';
    track.onload = this.step_func_done(function() {
      var got = track.track.cues[0].text;
      assert_equals(got, expected_current);
    });
  }, 'loading webvtt <track>');

  // XXX downloading seems hard to automate
  // <a href download>
  // <area href download>

  // submit forms
  function test_submit_form(tag, attr) {
    subsetTestByKey('submit', async_test, function(){
      var elm = document.createElement(tag);
      elm.setAttribute(attr, input_url_html);
      var form;
      var button;
      if (tag == 'form') {
        form = elm;
        button = document.createElement('button');
      } else {
        form = document.createElement('form');
        button = elm;
      }
      form.method = 'post';
      form.appendChild(button);
      var iframe = document.createElement('iframe');
      var id = 'test_submit_form_' + tag;
      iframe.name = id;
      form.target = id;
      button.type = 'submit';
      document.body.appendChild(form);
      document.body.appendChild(iframe);
      this.add_cleanup(function() {
        document.body.removeChild(form);
        document.body.removeChild(iframe);
      });
      button.click();
      iframe.onload = this.step_func_done(function() {
        var got = iframe.contentDocument.body.textContent;
        if (got == '') {
          return;
        }
        assert_equals(got, expected_current);
      });
    }, 'submit form <'+tag+' '+attr+'>');
  }

  'form action, input formaction, button formaction'.split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_submit_form(arr[0], arr[1]);
  });

  // <base href>
  subsetTestByKey('base-href', async_test, function() {
    var iframe = document.createElement('iframe');
    iframe.src = blank;
    document.body.appendChild(iframe);
    this.add_cleanup(function() {
      document.body.removeChild(iframe);
    });
    iframe.onload = this.step_func_done(function() {
      var doc = iframe.contentDocument;
      doc.write('<!doctype html><base href="'+input_url+'"><a href></a>');
      doc.close();
      var got_baseURI = doc.baseURI;
      assert_true(got_baseURI.indexOf(expected_current) > -1, msg(expected_current, got_baseURI), 'doc.baseURI');
      var got_a_href = doc.links[0].href;
      assert_true(got_a_href.indexOf(expected_current) > -1, msg(expected_current, got_a_href), 'a.href');
    });
  }, '<base href>');

  // XXX drag and drop (<a href> or <img src>) seems hard to automate

  // Worker()
  subsetTestByKey('workers', async_test, function() {
    var worker = new Worker(input_url_worker);
    worker.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_current);
    });
  }, 'Worker constructor');

  // SharedWorker()
  subsetTestByKey('workers', async_test, function() {
    var worker = new SharedWorker(input_url_sharedworker);
    worker.port.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_current);
    });
  }, 'SharedWorker constructor');

  // EventSource()
  subsetTestByKey('eventsource', async_test, function() {
    var source = new EventSource(input_url_eventstream);
    this.add_cleanup(function() {
      source.close();
    });
    source.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_current);
    });
  }, 'EventSource constructor');

  // EventSource#url
  subsetTestByKey('eventsource', test, function() {
    var source = new EventSource(input_url_eventstream);
    source.close();
    var got = source.url;
    assert_true(source.url.indexOf(expected_current) > -1, msg(expected_current, got));
  }, 'EventSource#url');

  // window.open
  subsetTestByKey('window-open', async_test, function() {
    var id = 'test_window_open';
    var iframe = document.createElement('iframe');
    iframe.name = id;
    document.body.appendChild(iframe);
    this.add_cleanup(function() {
      document.body.removeChild(iframe);
    });
    window.open(input_url_html, id);
    iframe.onload = this.step_func(function() {
      var got = iframe.contentDocument.body.textContent;
      if (got != "") {
        assert_equals(got, expected_current);
        this.done();
      }
    });
  }, 'window.open()');

  // a.search, area.search
  function test_hyperlink_search(tag) {
    subsetTestByKey('hyperlink-search', test, function() {
      var elm = document.createElement(tag);
      var input_arr = input_url_html.split('?');
      elm.href = input_arr[0];
      elm.search = '?' + input_arr[1];
      var got_href = elm.getAttribute('href');
      assert_true(got_href.indexOf(expected_utf8) > -1, 'href content attribute ' + msg(expected_utf8, got_href));
      var got_search = elm.search;
      assert_true(got_search.indexOf(expected_utf8) > -1, 'getting .search '+msg(expected_utf8, got_search));
    }, '<'+tag+'>.search');
  }
  'a, area'.split(', ').forEach(function(str) {
    test_hyperlink_search(str);
  });

  // history.pushState
  // history.replaceState
  function test_history(prop) {
    subsetTestByKey('history', async_test, function() {
      var url = input_url_html.replace('resources/', '');
      var iframe = document.createElement('iframe');
      iframe.src = blank;
      document.body.appendChild(iframe);
      this.add_cleanup(function() {
        document.body.removeChild(iframe);
      });
      iframe.onload = this.step_func_done(function() {
        // this should resolve against the iframe's URL
        // "Parse url, relative to the relevant settings object of history."
        // https://html.spec.whatwg.org/multipage/nav-history-apis.html#shared-history-push%2Freplace-state-steps
        iframe.contentWindow.history[prop](null, null, url);
        var got = iframe.contentWindow.location.href;
        assert_true(got.indexOf(expected_current) > -1, msg(expected_current, got));
        assert_not_equals(got.indexOf('/resources/'), -1, 'url was resolved against the test\'s URL');
      });
    }, 'history.'+prop);
  }

  'pushState, replaceState'.split(', ').forEach(function(str) {
    test_history(str);
  });

  // SVG
  var ns = {svg:'http://www.w3.org/2000/svg', xlink:'http://www.w3.org/1999/xlink'};
  // a
  subsetTestByKey('svg', async_test, function() {
    SVGAElement; // check support
    var iframe = document.createElement('iframe');
    var id = 'test_svg_a';
    iframe.name = id;
    var svg = document.createElementNS(ns.svg, 'svg');
    var a = document.createElementNS(ns.svg, 'a');
    a.setAttributeNS(ns.xlink, 'xlink:href', input_url_html);
    a.setAttribute('target', id);
    var span = document.createElement('span');
    a.appendChild(span);
    svg.appendChild(a);
    document.body.appendChild(iframe);
    document.body.appendChild(svg);
    this.add_cleanup(function() {
      document.body.removeChild(iframe);
      document.body.removeChild(svg);
    });
    span.click();
    iframe.onload = this.step_func_done(function() {
      var got = iframe.contentDocument.body.textContent;
      if (got != '') {
        assert_equals(got, expected_current);
      }
    });
  }, 'SVG <a>');

  // feImage, image, use
  function test_svg(func, tag) {
    subsetTestByKey('svg', async_test, function() {
      var uuid = token();
      var id = 'test_svg_'+tag;
      var svg = document.createElementNS(ns.svg, 'svg');
      var parent = func(svg, id);
      var elm = document.createElementNS(ns.svg, tag);
      elm.setAttributeNS(ns.xlink, 'xlink:href', stash_put + uuid + '#foo');
      parent.appendChild(elm);
      document.body.appendChild(svg);
      this.add_cleanup(function() {
        document.body.removeChild(svg);
      });
      poll_for_stash(this, uuid, expected_current);
    }, 'SVG <' + tag + '>');
  }

  [[function(svg, id) {
      SVGFEImageElement; // check support
      var filter = document.createElementNS(ns.svg, 'filter');
      filter.setAttribute('id', id);
      svg.appendChild(filter);
      var rect = document.createElementNS(ns.svg, 'rect');
      rect.setAttribute('filter', 'url(#'+id+')');
      svg.appendChild(rect);
      return filter;
    }, 'feImage'],
   [function(svg, id) { SVGImageElement; return svg; }, 'image'],
   [function(svg, id) { SVGUseElement; return svg; }, 'use']].forEach(function(arr) {
    test_svg(arr[0], arr[1]);
  });

  // UTF-8:
  // XHR
  subsetTestByKey('xhr', async_test, function() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', input_url_html);
    xhr.onload = this.step_func_done(function() {
      assert_equals(xhr.response, expected_current);
    });
    xhr.send();
  }, 'XMLHttpRequest#open()');

  // in a worker
  subsetTestByKey('workers', async_test, function() {
    var worker = new Worker(input_url_worker_importScripts);
    worker.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'importScripts() in a dedicated worker');

  subsetTestByKey('workers', async_test, function() {
    var worker = new Worker(input_url_worker_worker);
    worker.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'Worker() in a dedicated worker');

  subsetTestByKey('workers', async_test, function() {
    var worker = new SharedWorker(input_url_sharedworker_importScripts);
    worker.port.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'importScripts() in a shared worker');

  subsetTestByKey('workers', async_test, function() {
    var worker = new SharedWorker(input_url_sharedworker_worker);
    worker.port.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'Worker() in a shared worker');

  // WebSocket()
  subsetTestByKey('websocket', async_test, function() {
    var ws = new WebSocket('ws://{{host}}:{{ports[ws][0]}}/echo-query?\u00E5');
    this.add_cleanup(function() {
      ws.close();
    });
    ws.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'WebSocket constructor');

  // WebSocket#url
  subsetTestByKey('websocket', test, function() {
    var ws = new WebSocket('ws://{{host}}:{{ports[ws][0]}}/echo-query?\u00E5');
    ws.close();
    var got = ws.url;
    assert_true(ws.url.indexOf(expected_utf8) > -1, msg(expected_utf8, got));
  }, 'WebSocket#url');

  // CSS
  function test_css(tmpl, expected_cssom, encoding, use_style_element) {
    var desc = ['CSS', (use_style_element ? '<style>' : '<link> (' + encoding + ')'),  tmpl].join(' ');
    subsetTestByKey('css', async_test, function(){
      css_is_supported(tmpl, expected_cssom, this);
      var uuid = token();
      var id = 'test_css_' + uuid;
      var url = 'url(stash.py?q=%s&action=put&id=' + uuid + ')';
      tmpl = tmpl.replace(/<id>/g, id).replace(/<url>/g, url);
      var link;
      if (use_style_element) {
        link = document.createElement('style');
        link.textContent = tmpl.replace(/%s/g, '\u00E5').replace(/stash\.py/g, 'resources/stash.py');
      } else {
        link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = 'resources/css-tmpl.py?encoding='+encoding+'&tmpl='+encodeURIComponent(tmpl);
      }
      var div = document.createElement('div');
      div.id = id;
      div.textContent='x';
      document.head.appendChild(link);
      document.body.appendChild(div);
      this.add_cleanup(function() {
        document.head.removeChild(link);
        document.body.removeChild(div);
      });
      poll_for_stash(this, uuid, expected_utf8);
    }, desc);
  }

  // fail fast if the input doesn't parse into the expected cssom
  function css_is_supported(tmpl, expected_cssom, test_obj) {
    if (expected_cssom === null) {
      return;
    }
    var style = document.createElement('style');
    style.textContent = tmpl.replace(/<id>/g, 'x').replace(/<url>/g, 'url(data:,)');
    document.head.appendChild(style);
    test_obj.add_cleanup(function() {
      document.head.removeChild(style);
    });
    assert_equals(style.sheet.cssRules.length, expected_cssom.length, 'number of style rules');
    for (var i = 0; i < expected_cssom.length; ++i) {
      if (expected_cssom[i] === null) {
        continue;
      }
      assert_equals(style.sheet.cssRules[i].style.length, expected_cssom[i], 'number of declarations in style rule #'+i);
    }
  }

  [['#<id> { background-image:<url> }', [1] ],
   ['#<id> { border-image-source:<url> }', [1] ],
   ['#<id>::before { content:<url> }', [1] ],
   ['@font-face { font-family:<id>; src:<url> } #<id> { font-family:<id> }', [null, 1] ],
   ['#<id> { display:list-item; list-style-image:<url> }', [2] ],
   ['@import <url>;', null ],
   // XXX maybe cursor isn't suitable for automation here if browsers delay fetching it
   ['#<id> { cursor:<url>, pointer }', [1] ]].forEach(function(arr) {
    var input = arr[0];
    var expected_cssom = arr[1];
    var other_encoding = encoding == 'utf-8' ? 'windows-1252' : 'utf-8';
    test_css(input, expected_cssom, encoding);
    test_css(input, expected_cssom, other_encoding);
    test_css(input, expected_cssom, null, true);
   });

  // XXX maybe test if they become relevant:
  // binding (obsolete?)
  // aural: cue-after, cue-before, play-during (not implemented?)
  // hyphenate-resource (not implemented?)
  // image() (not implemented?)

  // <?xml-stylesheet?>
  subsetTestByKey('xml', async_test, function() {
    var iframe = document.createElement('iframe');
    iframe.src = input_url_xmlstylesheet_css;
    document.body.appendChild(iframe);
    this.add_cleanup(function() {
      document.body.removeChild(iframe);
    });
    iframe.onload = this.step_func_done(function() {
      assert_equals(iframe.contentDocument.firstChild.sheet.cssRules[0].style.content, '"' + expected_utf8 + '"');
    });
  }, '<?xml-stylesheet?> (CSS)');

  // new URL()
  subsetTestByKey('url', test, function() {
    var url = new URL('http://example.org/'+input_url);
    var expected = expected_utf8;
    assert_true(url.href.indexOf(expected) > -1, 'url.href '+msg(expected, url.href));
    assert_true(url.search.indexOf(expected) > -1, 'url.search '+msg(expected, url.search));
  }, 'URL constructor, url');

  subsetTestByKey('url', test, function() {
    var url = new URL('', 'http://example.org/'+input_url);
    var expected = expected_utf8;
    assert_true(url.href.indexOf(expected) > -1, 'url.href '+msg(expected, url.href));
    assert_true(url.search.indexOf(expected) > -1, 'url.search '+msg(expected, url.search));
  }, 'URL constructor, base');

  // Test different schemes
  function test_scheme(url, utf8) {
    subsetTestByKey('scheme', test, function() {
      var a = document.createElement('a');
      a.setAttribute('href', url);
      var got = a.href;
      var expected = utf8 ? expected_utf8 : expected_current;
      assert_true(got.indexOf(expected) != -1, msg(expected, got));
    }, 'Scheme ' + url.split(':')[0] + ' (getting <a>.href)');
  }

  var test_scheme_urls = ['ftp://example.invalid/?x=\u00E5',
                          'file:///?x=\u00E5',
                          'http://example.invalid/?x=\u00E5',
                          'https://example.invalid/?x=\u00E5',
                         ];

  var test_scheme_urls_utf8 = ['ws://example.invalid/?x=\u00E5',
                               'wss://example.invalid/?x=\u00E5',
                               'gopher://example.invalid/?x=\u00E5',
                               'mailto:example@invalid?x=\u00E5',
                               'data:text/plain;charset='+encoding+',?x=\u00E5',
                               'javascript:"?x=\u00E5"',
                               'ftps://example.invalid/?x=\u00E5',
                               'httpbogus://example.invalid/?x=\u00E5',
                               'bitcoin:foo?x=\u00E5',
                               'geo:foo?x=\u00E5',
                               'im:foo?x=\u00E5',
                               'irc:foo?x=\u00E5',
                               'ircs:foo?x=\u00E5',
                               'magnet:foo?x=\u00E5',
                               'mms:foo?x=\u00E5',
                               'news:foo?x=\u00E5',
                               'nntp:foo?x=\u00E5',
                               'sip:foo?x=\u00E5',
                               'sms:foo?x=\u00E5',
                               'smsto:foo?x=\u00E5',
                               'ssh:foo?x=\u00E5',
                               'tel:foo?x=\u00E5',
                               'urn:foo?x=\u00E5',
                               'webcal:foo?x=\u00E5',
                               'wtai:foo?x=\u00E5',
                               'xmpp:foo?x=\u00E5',
                               'web+http:foo?x=\u00E5',
                              ];

  test_scheme_urls.forEach(function(url) {
    test_scheme(url);
  });

  test_scheme_urls_utf8.forEach(function(url) {
    test_scheme(url, true);
  });

  done();
};
