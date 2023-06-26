This directory contains a number of tests from
[web-platform-tests](https://github.com/web-platform-tests/wpt) at
77585330fd7da01392aec01cf5fed7aa22597180, chosen from the files processed by the manifest script.

These files are split into two directories:

 * `weighted`, a set of 15 tests curated from a random weighted sample of 30, weighted by parse
   time as of html5lib 1.0.1. The curation was performed primarily as many of the slowest files are
   very similar and therefore provide little extra coverage while it is relatively probable both
   with be chosen. This provides a set of files which significantly contribute to the manifest
   generation time.

 * `random`, a further set of 15 tests, this time a random unweighted sample of 15. This provides a
   set of files much closer to the average file in WPT.

The files are sourced from the following:

`weighted`:

 * `css/compositing/test-plan/test-plan.src.html`
 * `css/css-flexbox/align-content-wrap-002.html`
 * `css/css-grid/grid-definition/grid-auto-fill-rows-001.html`
 * `css/css-grid/masonry.tentative/masonry-item-placement-006.html`
 * `css/css-images/image-orientation/reference/image-orientation-from-image-content-images-ref.html`
 * `css/css-position/position-sticky-table-th-bottom-ref.html`
 * `css/css-text/white-space/pre-float-001.html`
 * `css/css-ui/resize-004.html`
 * `css/css-will-change/will-change-abspos-cb-001.html`
 * `css/filter-effects/filter-turbulence-invalid-001.html`
 * `css/vendor-imports/mozilla/mozilla-central-reftests/css21/pagination/moz-css21-table-page-break-inside-avoid-2.html`
 * `encoding/legacy-mb-tchinese/big5/big5_chars_extra.html`
 * `html/canvas/element/compositing/2d.composite.image.destination-over.html`
 * `html/semantics/embedded-content/the-canvas-element/toBlob.png.html`
 * `referrer-policy/4K-1/gen/top.http-rp/unsafe-url/fetch.http.html`

`random`:

 * `content-security-policy/frame-ancestors/frame-ancestors-self-allow.html`
 * `css/css-backgrounds/reference/background-origin-007-ref.html`
 * `css/css-fonts/idlharness.html`
 * `css/css-position/static-position/htb-ltr-ltr.html`
 * `css/vendor-imports/mozilla/mozilla-central-reftests/css21/pagination/moz-css21-float-page-break-inside-avoid-6.html`
 * `css/vendor-imports/mozilla/mozilla-central-reftests/shapes1/shape-outside-content-box-002.html`
 * `encoding/legacy-mb-korean/euc-kr/euckr-encode-form.html`
 * `html/browsers/browsing-the-web/unloading-documents/beforeunload-on-history-back-1.html`
 * `html/browsers/the-window-object/apis-for-creating-and-navigating-browsing-contexts-by-name/non_automated/001.html`
 * `html/editing/dnd/overlay/heavy-styling-005.html`
 * `html/rendering/non-replaced-elements/lists/li-type-unsupported-ref.html`
 * `html/semantics/grouping-content/the-dl-element/grouping-dl.html`
 * `trusted-types/worker-constructor.https.html`
 * `webvtt/rendering/cues-with-video/processing-model/selectors/cue/background_shorthand_css_relative_url.html`
 * `IndexedDB/idbindex_get8.htm`
