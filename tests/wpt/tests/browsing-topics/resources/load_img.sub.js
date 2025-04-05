// Poll the server for the test result.
async function get_stashed_topics_header(url) {
  for (let i = 0; i < 30; ++i) {
      const response = await fetch(url + '&query');
      let stashed_topics_header = await response.text();

      if (!stashed_topics_header || (stashed_topics_header === 'NO_PREVIOUS_REQUEST')) {
          await new Promise(resolve => step_timeout(resolve, 100));
          continue;
      }
      return stashed_topics_header;
  }
  assert_true(false, 'timeout');
}

// Load an image and poll for the topics header that
// check-topics-request-header-in-img.py should stash.
function load_topics_image(has_browsing_topics_attribute, is_same_origin) {
    let stash_id = token();
    const sameOriginSrc = `/browsing-topics/resources/` +
        `check-topics-request-header-in-img.py?token=${stash_id}`;
    const crossOriginSrc = 'https://{{domains[www]}}:{{ports[https][0]}}' +
        sameOriginSrc;

    const url = is_same_origin ? sameOriginSrc : crossOriginSrc

    let image = document.createElement('img');
    image.src = url;

    if (has_browsing_topics_attribute) {
      image.browsingTopics = true;
    }

    image.decode().then(() => {
       document.body.appendChild(image);
     });

    return get_stashed_topics_header(url);
}