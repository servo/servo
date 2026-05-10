// https://html.spec.whatwg.org/multipage/images.html#updating-the-image-data
// Under Step 27, after fetch, step 3: Add the image to the list of available
// images using the key key, with the ignore higher-layer caching flag set.
//
// Step 7.4: If the list of available images contains an entry for key, then:
// Step 7.4.4: Set the current request to a new image request whose image data
// is that of the entry and whose state is completely available.

promise_test(async () => {
  const img = document.createElement("img");
  img.src = "resources/image-alternate-no-store.py";
  await img.decode();
  let width = img.naturalWidth;

  img.src = img.src;
  await img.decode();
  assert_equals(width, img.naturalWidth, "The image size should be the same");
}, "Reassigning the same src value should not trigger an extra image fetch");
