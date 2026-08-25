'use strict';

async function loadBfCacheTestHelperResources() {
  await loadScript('/common/utils.js');
  await loadScript('/common/dispatcher/dispatcher.js');
  await loadScript(
      '/html/browsers/browsing-the-web/back-forward-cache/resources/helper.sub.js');
}
await loadBfCacheTestHelperResources();

// Runs BFCache tests for embed elements, specifically <embed> and <object>.
// 1. Attaches the target element to first page.
// 2. Navigates away, then back via bfcache if this case is supported by the
//    browser.
// @param {Object}  testCase - The target element's attributes to test with.
export function runBfcacheTestForEmbeds(testCase) {
  assert_implements(runBfcacheTest, '`runBfcacheTest()` is unavailable.');
  assert_implements(originSameOrigin, '`originSameOrigin` is unavailable.');

  const tags = [
    {'name': 'embed', 'srcAttr': 'src'},
    {'name': 'object', 'srcAttr': 'data'},
  ];
  for (const tag of tags) {
    runBfcacheTest(
        {
          targetOrigin: originSameOrigin,
          shouldBeCached: true,
          funcBeforeNavigation: (tag, attrs) => {
            let e = document.createElement(tag.name);
            // Only sets defined attributes to match the intended test behavior
            // like embedded-type-only.html test.
            if ('type' in attrs) {
              e.type = attrs.type;
            }
            if ('src' in attrs) {
              e[tag.srcAttr] = attrs.src;
            }
            document.body.append(e);
          },
          argsBeforeNavigation: [tag, testCase]
        },
        `Page with <${tag.name} ` +
            `type=${testCase.type} ${tag.srcAttr}=${testCase.src}>`);
  }
}
