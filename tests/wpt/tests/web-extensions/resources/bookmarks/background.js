const testTitle = 'WPT Test Bookmark';
const testUrl = 'https://example.com/';

browser.test.runTests([
  /**
   * Tests presence of `browser.bookmarks.ROOT_NODE_ID`.
   */
  function testBookmarksRootNodeId() {
    browser.test.assertTrue(
        'ROOT_NODE_ID' in browser.bookmarks,
        'browser.bookmarks should contain ROOT_NODE_ID property');
    browser.test.assertEq(typeof browser.bookmarks.ROOT_NODE_ID, 'string',
                          'browser.bookmarks.ROOT_NODE_ID should be a string');
  },

  /**
   * Tests `browser.bookmarks.create` and `browser.bookmarks.get`.
   */
  async function testBookmarksCreateAndGet() {
    const node = await browser.bookmarks.create({
      title: testTitle,
      url: testUrl,
    });
    browser.test.assertEq(typeof node.id, 'string',
                          'Created bookmark ID should be a string');
    browser.test.assertEq(node.title, testTitle, 'Bookmark title should match');
    browser.test.assertEq(node.url, testUrl, 'Bookmark URL should match');

    const retrievedNodes = await browser.bookmarks.get(node.id);
    browser.test.assertEq(retrievedNodes.length, 1,
                          'get should return array with 1 node');
    browser.test.assertEq(retrievedNodes[0].id, node.id,
                          'Retrieved ID should match');
    browser.test.assertEq(retrievedNodes[0].title, testTitle,
                          'Retrieved title should match');
    await browser.bookmarks.remove(node.id);
  },

  /**
   * Tests `browser.bookmarks.update`.
   */
  async function testBookmarksUpdate() {
    const node = await browser.bookmarks.create({
      title: testTitle,
      url: testUrl,
    });
    const updatedTitle = 'WPT Test Bookmark Updated';
    const updatedNode = await browser.bookmarks.update(node.id, {
      title: updatedTitle,
    });
    browser.test.assertEq(updatedNode.id, node.id,
                          'Updated node ID should match');
    browser.test.assertEq(updatedNode.title, updatedTitle,
                          'Updated title should match');
    await browser.bookmarks.remove(node.id);
  },

  /**
   * Tests `browser.bookmarks.search`.
   */
  async function testBookmarksSearch() {
    const node = await browser.bookmarks.create({
      title: testTitle,
      url: testUrl,
    });
    const searchResults = await browser.bookmarks.search({url: testUrl});
    browser.test.assertTrue(searchResults.length > 0,
                            'Search by URL should return results');
    const match = searchResults.find((r) => r.id === node.id);
    browser.test.assertTrue(
        Boolean(match),
        'Search results should include the created bookmark ID');
    browser.test.assertEq(match.title, testTitle,
                          'Search result title should match');
    await browser.bookmarks.remove(node.id);
  },

  /**
   * Tests `browser.bookmarks.getTree`.
   */
  async function testBookmarksGetTree() {
    const tree = await browser.bookmarks.getTree();
    browser.test.assertTrue(Array.isArray(tree),
                            'getTree should return an array');
    browser.test.assertTrue(tree.length > 0,
                            'Bookmark tree should contain root nodes');
  },

  /**
   * Tests `browser.bookmarks.remove`.
   */
  async function testBookmarksRemove() {
    const node = await browser.bookmarks.create({
      title: testTitle,
      url: testUrl,
    });
    await browser.bookmarks.remove(node.id);
    try {
      await browser.bookmarks.get(node.id);
      browser.test.fail('get should reject for removed bookmark');
    } catch (e) {
      // Expected rejection.
    }
  },

  /**
   * Tests `browser.bookmarks.create` error cases.
   */
  function testBookmarksErrorCases() {
    // Verify `create` throws when passed invalid object types.
    browser.test.assertThrows(() => browser.bookmarks.create('invalid'));
  }
]);
