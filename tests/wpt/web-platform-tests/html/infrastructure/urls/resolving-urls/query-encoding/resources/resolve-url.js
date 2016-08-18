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
    'windows-1251':'%3F'
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
                  setTimeout(poll, 200);
              } else {
                  assert_equals(xhr.response, expected);
                  test_obj.done();
              }
          });
          xhr.send();
      })
      setTimeout(poll, 200);
  }

  // background attribute, check with getComputedStyle
  function test_background(tag) {
    var spec_url = 'https://html.spec.whatwg.org/multipage/multipage/rendering.html';
    spec_url += tag == 'body' ? '#the-page' : '#tables';
    test(function() {
      var elm = document.createElement(tag);
      document.body.appendChild(elm);
      this.add_cleanup(function() {
        document.body.removeChild(elm);
      });
      elm.setAttribute('background', input_url_png);
      var got = getComputedStyle(elm).backgroundImage;
      assert_true(got.indexOf(expected_current) > -1, msg(expected_current, got));
    }, 'getComputedStyle <'+tag+' background>',
    {help:spec_url});
  }

  'body, table, thead, tbody, tfoot, tr, td, th'.split(', ').forEach(function(str) {
    test_background(str);
  });

  // get a reflecting IDL attributes whose content attribute takes a URL or a list of space-separated URLs
  function test_reflecting(tag, attr, idlAttr, multiple) {
    idlAttr = idlAttr || attr;
    var input = input_url_html;
    if (multiple) {
      input += ' ' + input;
    }
    test(function() {
      var elm = document.createElement(tag);
      assert_true(idlAttr in elm, idlAttr + ' is not supported');
      elm.setAttribute(attr, input);
      var got = elm[idlAttr];
      assert_true(got.indexOf(expected_current) > -1, msg(expected_current, got));
    }, 'Getting <'+tag+'>.'+idlAttr + (multiple ? ' (multiple URLs)' : ''),
    {help:'https://html.spec.whatwg.org/multipage/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes'});
  }

  ('iframe src, a href, base href, link href, img src, embed src, object data, track src, video src, audio src, input src, form action, ' +
  'input formaction formAction, button formaction formAction, menuitem icon, script src').split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_reflecting(arr[0], arr[1], arr[2]);
  });

  'a ping'.split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_reflecting(arr[0], arr[1], arr[2], true);
  });

  function setup_navigation(elm, iframe, id, test_obj) {
    iframe.name = id;
    elm.target = id;
    elm.setAttribute('href', input_url_html);
    document.body.appendChild(iframe);
    document.body.appendChild(elm);
    test_obj.add_cleanup(function() {
      document.body.removeChild(iframe);
      document.body.removeChild(elm);
    });
  }

  // follow hyperlink
  function test_follow_link(tag) {
    async_test(function() {
      var elm = document.createElement(tag);
      var iframe = document.createElement('iframe');
      setup_navigation(elm, iframe, 'test_follow_link_'+tag, this);
      iframe.onload = this.step_func_done(function() { // when the page navigated to has loaded
        assert_equals(iframe.contentDocument.body.textContent, expected_current);
      });
      // follow the hyperlink
      elm.click();
      // check that navigation succeeded by ...??? XXX
    }, 'follow hyperlink <'+tag+' href>',
    {help:'https://html.spec.whatwg.org/multipage/multipage/links.html#following-hyperlinks'});
  }

  'a, area, link'.split(', ').forEach(function(str) {
    test_follow_link(str);
  });

  // follow hyperlink with ping attribute
  function test_follow_link_ping(tag) {
    async_test(function() {
      var uuid = token();
      var elm = document.createElement(tag);
      // check if ping is supported
      assert_true('ping' in elm, 'ping not supported');
      elm.setAttribute('ping', stash_put + uuid);
      var iframe = document.createElement('iframe');
      setup_navigation(elm, iframe, 'test_follow_link_ping_'+tag, this);
      // follow the hyperlink
      elm.click();
      // check that navigation succeeded by ...??? XXX
      // check that the right URL was requested for the ping
      poll_for_stash(this, uuid, expected_current);
    }, 'hyperlink auditing <'+tag+' ping>',
    {help:'https://html.spec.whatwg.org/multipage/multipage/links.html#hyperlink-auditing'});
  }

  'a, area'.split(', ').forEach(function(str) {
    test_follow_link_ping(str);
  });

  // navigating with meta refresh
  async_test(function() {
    var iframe = document.createElement('iframe');
    iframe.src = blank;
    document.body.appendChild(iframe);
    this.add_cleanup(function() {
      document.body.removeChild(iframe);
    });
    iframe.onload = this.step_func_done(function() {
      var doc = iframe.contentDocument;
      var got = doc.body.textContent;
      if (got == '') {
        doc.write('<meta http-equiv=refresh content="0; URL='+input_url_html+'">REFRESH');
        doc.close();
        return;
      }
      assert_equals(got, expected_current);
    });
  }, 'meta refresh',
  {help:'https://html.spec.whatwg.org/multipage/multipage/semantics.html#attr-meta-http-equiv-refresh'});

  // loading html (or actually svg to support <embed>)
  function test_load_nested_browsing_context(tag, attr, spec_url) {
    async_test(function() {
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

    }, 'load nested browsing context <'+tag+' '+attr+'>',
    {help:spec_url});
  }

  spec_url_load_nested_browsing_context = {
    frame:'https://html.spec.whatwg.org/multipage/multipage/obsolete.html#process-the-frame-attributes',
    iframe:'https://html.spec.whatwg.org/multipage/multipage/the-iframe-element.html#process-the-iframe-attributes',
    object:'https://html.spec.whatwg.org/multipage/multipage/the-iframe-element.html#the-object-element',
    embed:'https://html.spec.whatwg.org/multipage/multipage/the-iframe-element.html#the-embed-element-setup-steps'
  };

  'frame src, iframe src, object data, embed src'.split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_load_nested_browsing_context(arr[0], arr[1], spec_url_load_nested_browsing_context[arr[0]]);
  });

  // loading css with <link>
  async_test(function() {
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
  }, 'loading css <link>',
  {help:['https://html.spec.whatwg.org/multipage/multipage/semantics.html#the-link-element',
         'https://html.spec.whatwg.org/multipage/multipage/semantics.html#styling']});

  // loading js
  async_test(function() {
    var elm = document.createElement('script');
    elm.src = input_url_js + '&var=test_load_js_got';
    document.head.appendChild(elm); // no cleanup
    elm.onload = this.step_func_done(function() {
      assert_equals(window.test_load_js_got, expected_current);
    });
  }, 'loading js <script>',
  {help:'https://html.spec.whatwg.org/multipage/multipage/scripting-1.html#prepare-a-script'});

  // loading image
  function test_load_image(tag, attr, spec_url) {
    async_test(function() {
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
    }, 'loading image <'+tag+' '+attr+'>',
    {help:spec_url});
  }

  var query_to_image_width = {
    '%E5':1,
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
    img:'https://html.spec.whatwg.org/multipage/multipage/embedded-content-1.html#update-the-image-data',
    embed:'https://html.spec.whatwg.org/multipage/multipage/the-iframe-element.html#the-embed-element-setup-steps',
    object:'https://html.spec.whatwg.org/multipage/multipage/the-iframe-element.html#the-object-element',
    input:'https://html.spec.whatwg.org/multipage/multipage/states-of-the-type-attribute.html#image-button-state-(type=image)',
    video:'https://html.spec.whatwg.org/multipage/multipage/the-video-element.html#poster-frame'
  };

  'img src, embed src, object data, input src, video poster'.split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_load_image(arr[0], arr[1], spec_url_load_image[arr[0]]);
  });

  // XXX test <img srcset> or its successor
  // <menuitem icon> could also be tested but the spec doesn't require it to be loaded...

  // loading video
  function test_load_video(tag, use_source_element) {
    async_test(function() {
      var elm = document.createElement(tag);
      var video_ext = '';
      if (elm.canPlayType('video/ogg; codecs="theora,flac"')) {
        video_ext = 'ogv';
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
    }, 'loading video <'+tag+'>' + (use_source_element ? '<source>' : ''),
    {help:'https://html.spec.whatwg.org/multipage/multipage/the-video-element.html#concept-media-load-algorithm'});
  }

  var query_to_video_duration = {
    '%E5':3,
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
  async_test(function() {
    var video = document.createElement('video');
    var track = document.createElement('track');
    video.appendChild(track);
    track.src = input_url_webvtt;
    track.track.mode = 'showing';
    track.onload = this.step_func_done(function() {
      var got = track.track.cues[0].text;
      assert_equals(got, expected_current);
    });
  }, 'loading webvtt <track>',
  {help:'https://html.spec.whatwg.org/multipage/multipage/the-video-element.html#track-url'});

  // XXX downloading seems hard to automate
  // <a href download>
  // <area href download>

  // submit forms
  function test_submit_form(tag, attr) {
    async_test(function(){
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
    }, 'submit form <'+tag+' '+attr+'>',
    {help:'https://html.spec.whatwg.org/multipage/multipage/association-of-controls-and-forms.html#concept-form-submit'});
  }

  'form action, input formaction, button formaction'.split(', ').forEach(function(str) {
    var arr = str.split(' ');
    test_submit_form(arr[0], arr[1]);
  });

  // <base href>
  async_test(function() {
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
  }, '<base href>',
  {help:['https://html.spec.whatwg.org/multipage/multipage/semantics.html#set-the-frozen-base-url',
  'https://dom.spec.whatwg.org/#dom-node-baseuri',
  'https://html.spec.whatwg.org/multipage/multipage/text-level-semantics.html#the-a-element']});

  // XXX drag and drop (<a href> or <img src>) seems hard to automate

  // Worker()
  async_test(function() {
    var worker = new Worker(input_url_worker);
    worker.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_current);
    });
  }, 'Worker constructor',
  {help:'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-worker'});

  // SharedWorker()
  async_test(function() {
    var worker = new SharedWorker(input_url_sharedworker);
    worker.port.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_current);
    });
  }, 'SharedWorker constructor',
  {help:'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-sharedworker'});

  // EventSource()
  async_test(function() {
    var source = new EventSource(input_url_eventstream);
    this.add_cleanup(function() {
      source.close();
    });
    source.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_current);
    });
  }, 'EventSource constructor',
  {help:'https://html.spec.whatwg.org/multipage/multipage/comms.html#dom-eventsource'});

  // EventSource#url
  test(function() {
    var source = new EventSource(input_url_eventstream);
    source.close();
    var got = source.url;
    assert_true(source.url.indexOf(expected_current) > -1, msg(expected_current, got));
  }, 'EventSource#url',
  {help:'https://html.spec.whatwg.org/multipage/multipage/comms.html#dom-eventsource'});

  // XMLDocument#load()
  async_test(function() {
    var doc = document.implementation.createDocument(null, "x");
    doc.load(input_url_svg);
    doc.onload = this.step_func_done(function() {
      assert_equals(doc.documentElement.textContent, expected_current);
    });
  }, 'XMLDocument#load()',
  {help:'https://html.spec.whatwg.org/multipage/multipage/dom.html#dom-xmldocument-load'});

  // window.open
  async_test(function() {
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
  }, 'window.open()',
  {help:'https://html.spec.whatwg.org/multipage/multipage/browsers.html#dom-open'});

  // location
  function test_location(func, desc) {
    async_test(function() {
      var iframe = document.createElement('iframe');
      document.body.appendChild(iframe);
      this.add_cleanup(function() {
        document.body.removeChild(iframe);
      });
      func(iframe.contentWindow, input_url_html);
      iframe.onload = this.step_func(function() {
        var got = iframe.contentDocument.body.textContent;
        if (got != '') {
          assert_equals(got, expected_current);
          this.done();
        }
      });
    }, desc,
    {help:'https://html.spec.whatwg.org/multipage/multipage/history.html#the-location-interface'});
  }
  [[function(win, input) { win.location = input; }, "location [PutForwards]"],
   [function(win, input) { win.location.assign(input); }, "location.assign()"],
   [function(win, input) { win.location.replace(input); }, "location.replace()"],
   [function(win, input) { win.location.href = input; }, "location.href"]].forEach(function(arr) {
    test_location(arr[0], arr[1]);
  });

  // location.search
  async_test(function() {
    var iframe = document.createElement('iframe');
    iframe.src = input_url_html;
    document.body.appendChild(iframe);
    this.add_cleanup(function() {
      document.body.removeChild(iframe);
    });
    var i = 0;
    iframe.onload = this.step_func(function() {
      i++;
      if (i == 1) {
        iframe.contentWindow.location.search = '?' + input_url_html.split('?')[1] + '&other=foobar';
      } else {
        var got = iframe.contentDocument.body.textContent;
        assert_equals(got, expected_current);
        this.done();
      }
    });
  }, 'location.search',
  {help:['https://html.spec.whatwg.org/multipage/multipage/history.html#the-location-interface',
         'http://url.spec.whatwg.org/#dom-url-search']});

  // a.search, area.search
  function test_hyperlink_search(tag) {
    test(function() {
      var elm = document.createElement(tag);
      var input_arr = input_url_html.split('?');
      elm.href = input_arr[0];
      elm.search = '?' + input_arr[1];
      var got_href = elm.getAttribute('href');
      assert_true(got_href.indexOf(expected_current) > -1, 'href content attribute ' + msg(expected_current, got_href));
      var got_search = elm.search;
      assert_true(got_search.indexOf(expected_current) > -1, 'getting .search '+msg(expected_current, got_search));
    }, '<'+tag+'>.search',
    {help:['https://html.spec.whatwg.org/multipage/multipage/text-level-semantics.html#the-'+tag+'-element',
           'http://url.spec.whatwg.org/#dom-url-search']});
  }
  'a, area'.split(', ').forEach(function(str) {
    test_hyperlink_search(str);
  });

  // history.pushState
  // history.replaceState
  function test_history(prop) {
    async_test(function() {
      var iframe = document.createElement('iframe');
      iframe.src = blank;
      document.body.appendChild(iframe);
      this.add_cleanup(function() {
        document.body.removeChild(iframe);
      });
      iframe.onload = this.step_func_done(function() {
        iframe.contentWindow.history[prop](null, null, input_url_html); // this should resolve against the test's URL, not the iframe's URL
        var got = iframe.contentWindow.location.href;
        assert_true(got.indexOf(expected_current) > -1, msg(expected_current, got));
        assert_equals(got.indexOf('/resources/resources/'), -1, 'url was resolved against the iframe\'s URL instead of the settings object\'s API base URL');
      });
    }, 'history.'+prop,
    {help:'https://html.spec.whatwg.org/multipage/multipage/history.html#dom-history-'+prop.toLowerCase()});
  }

  'pushState, replaceState'.split(', ').forEach(function(str) {
    test_history(str);
  });

  // SVG
  var ns = {svg:'http://www.w3.org/2000/svg', xlink:'http://www.w3.org/1999/xlink'};
  // a
  async_test(function() {
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
    async_test(function() {
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
    }, 'SVG <' + tag + '>',
    {help:'https://www.w3.org/Bugs/Public/show_bug.cgi?id=24148'});
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
  async_test(function() {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', input_url_html);
    xhr.onload = this.step_func_done(function() {
      assert_equals(xhr.response, expected_utf8);
    });
    xhr.send();
  }, 'XMLHttpRequest#open()',
  {help:'https://xhr.spec.whatwg.org/#the-open()-method'});

  // in a worker
  async_test(function() {
    var worker = new Worker(input_url_worker_importScripts);
    worker.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'importScripts() in a dedicated worker',
  {help:['https://html.spec.whatwg.org/multipage/multipage/workers.html#set-up-a-worker-script-settings-object',
         'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-workerglobalscope-importscripts']});

  async_test(function() {
    var worker = new Worker(input_url_worker_worker);
    worker.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'Worker() in a dedicated worker',
  {help:['https://html.spec.whatwg.org/multipage/multipage/workers.html#set-up-a-worker-script-settings-object',
         'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-worker']});

  async_test(function() {
    var worker = new Worker(input_url_worker_sharedworker);
    worker.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'SharedWorker() in a dedicated worker',
  {help:['https://html.spec.whatwg.org/multipage/multipage/workers.html#set-up-a-worker-script-settings-object',
         'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-sharedworker']});

  async_test(function() {
    var worker = new SharedWorker(input_url_sharedworker_importScripts);
    worker.port.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'importScripts() in a shared worker',
  {help:['https://html.spec.whatwg.org/multipage/multipage/workers.html#set-up-a-worker-script-settings-object',
         'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-workerglobalscope-importscripts']});

  async_test(function() {
    var worker = new SharedWorker(input_url_sharedworker_worker);
    worker.port.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'Worker() in a shared worker',
  {help:['https://html.spec.whatwg.org/multipage/multipage/workers.html#set-up-a-worker-script-settings-object',
         'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-worker']});

  async_test(function() {
    var worker = new SharedWorker(input_url_sharedworker_sharedworker);
    worker.port.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'SharedWorker() in a shared worker',
  {help:['https://html.spec.whatwg.org/multipage/multipage/workers.html#set-up-a-worker-script-settings-object',
         'https://html.spec.whatwg.org/multipage/multipage/workers.html#dom-sharedworker']});

  // WebSocket()
  async_test(function(){
    var ws = new WebSocket('ws://{{host}}:{{ports[ws][0]}}/echo-query?\u00E5');
    this.add_cleanup(function() {
      ws.close();
    });
    ws.onmessage = this.step_func_done(function(e) {
      assert_equals(e.data, expected_utf8);
    });
  }, 'WebSocket constructor',
  {help:'https://html.spec.whatwg.org/multipage/multipage/network.html#parse-a-websocket-url\'s-components'});

  // WebSocket#url
  test(function(){
    var ws = new WebSocket('ws://{{host}}:{{ports[ws][0]}}/echo-query?\u00E5');
    ws.close();
    var got = ws.url;
    assert_true(ws.url.indexOf(expected_utf8) > -1, msg(expected_utf8, got));
  }, 'WebSocket#url',
  {help:'https://html.spec.whatwg.org/multipage/multipage/network.html#dom-websocket-url'});

  // Parsing cache manifest
  function test_cache_manifest(mode) {
    async_test(function() {
      var iframe = document.createElement('iframe');
      var uuid = token();
      iframe.src = 'resources/page-using-manifest.py?id='+uuid+'&encoding='+encoding+'&mode='+mode;
      document.body.appendChild(iframe);
      this.add_cleanup(function() {
        document.body.removeChild(iframe);
      });
      poll_for_stash(this, uuid, expected_utf8);
    }, 'Parsing cache manifest (' + mode + ')',
    {help:'https://html.spec.whatwg.org/multipage/multipage/offline.html#parse-a-manifest'});
  }

  'CACHE, FALLBACK, NETWORK'.split(', ').forEach(function(str) {
    test_cache_manifest(str);
  });

  // CSS
  function test_css(tmpl, expected_cssom, encoding, use_style_element) {
    var desc = ['CSS', (use_style_element ? '<style>' : '<link> (' + encoding + ')'),  tmpl].join(' ');
    async_test(function(){
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
    }, desc,
    {help:'https://www.w3.org/Bugs/Public/show_bug.cgi?id=23968'});
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
  async_test(function() {
    var iframe = document.createElement('iframe');
    iframe.src = input_url_xmlstylesheet_css;
    document.body.appendChild(iframe);
    this.add_cleanup(function() {
      document.body.removeChild(iframe);
    });
    iframe.onload = this.step_func_done(function() {
      assert_equals(iframe.contentDocument.firstChild.sheet.cssRules[0].style.content, '"' + expected_utf8 + '"');
    });
  }, '<?xml-stylesheet?> (CSS)',
  {help:'http://dev.w3.org/csswg/cssom/#requirements-on-user-agents-implementing-the-xml-stylesheet-processing-instruction'});

  // new URL()
  test(function() {
    var url = new URL('http://example.org/'+input_url);
    var expected = expected_utf8;
    assert_true(url.href.indexOf(expected) > -1, 'url.href '+msg(expected, url.href));
    assert_true(url.search.indexOf(expected) > -1, 'url.search '+msg(expected, url.search));
  }, 'URL constructor, url',
  {help:'http://url.spec.whatwg.org/#dom-url'});

  test(function() {
    var url = new URL('', 'http://example.org/'+input_url);
    var expected = expected_utf8;
    assert_true(url.href.indexOf(expected) > -1, 'url.href '+msg(expected, url.href));
    assert_true(url.search.indexOf(expected) > -1, 'url.search '+msg(expected, url.search));
  }, 'URL constructor, base',
  {help:'http://url.spec.whatwg.org/#dom-url'});

  // Test different schemes
  function test_scheme(url, utf8) {
    test(function() {
      var a = document.createElement('a');
      a.setAttribute('href', url);
      var got = a.href;
      var expected = utf8 ? expected_utf8 : expected_current;
      assert_true(got.indexOf(expected) != -1, msg(expected, got));
    }, 'Scheme ' + url.split(':')[0] + ' (getting <a>.href)');
  }

  var test_scheme_urls = ['ftp://example.invalid/?x=\u00E5',
                          'file:///?x=\u00E5',
                          'gopher://example.invalid/?x=\u00E5',
                          'http://example.invalid/?x=\u00E5',
                          'https://example.invalid/?x=\u00E5',
                         ];

  var test_scheme_urls_utf8 = ['ws://example.invalid/?x=\u00E5',
                               'wss://example.invalid/?x=\u00E5',
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
