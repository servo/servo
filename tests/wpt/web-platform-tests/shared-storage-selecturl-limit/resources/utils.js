// META: script=/fenced-frame/resources/utils.js

// Generate a list of URLs to use as input to sharedStorage.selectURL.
// @param {integer} size - The number of URLs to generate
// @param {string} pathPrefix - Prefix of the relative URL to use
// @param {string list} keylist - The list of key UUIDs to be used. Note that
//                                order matters when extracting the keys
// @return {SharedStorageUrlWithMetadata list} - A list of
//                                              SharedStorageUrlWithMetadata
//                                              dictionaries whose "url"
//                                              values have `keylist` appended
//                                              to their `searchParams`
function generateUrls(size, pathPrefix, keylist) {
  return new Array(size).fill(0).map((e, i) => {
    return {
      url: generateURL(pathPrefix + i.toString() + '.html', keylist)
    }
  });
}
