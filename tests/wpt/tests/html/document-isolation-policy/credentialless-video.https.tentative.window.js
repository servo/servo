// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const cookie_key = "dip_credentialless_image";
const cookie_same_origin = "same_origin";
const cookie_cross_origin = "cross_origin";

promise_setup(async test => {
  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin +
      cookie_same_site_none),
    setCookie(cross_origin, cookie_key, cookie_cross_origin +
      cookie_same_site_none),
  ]);
}, "Setup cookies");

const videoTest = function(description, origin, mode, expected_cookie) {
  promise_test(async test => {
    const video_token = token();

    let video = document.createElement("video");
    video.src = showRequestHeaders(origin, video_token);
    video.autoplay = true;
    if (mode)
      video.crossOrigin = mode;
    document.body.appendChild(video);

    const headers = JSON.parse(await receive(video_token));

    assert_equals(parseCookies(headers)[cookie_key], expected_cookie);
  }, `video ${description}`)
};

// Same-origin request always contains Cookies:
videoTest("same-origin + undefined",
  same_origin, undefined, cookie_same_origin);
videoTest("same-origin + anonymous",
  same_origin, 'anonymous', cookie_same_origin);
videoTest("same-origin + use-credentials",
  same_origin, 'use-credentials', cookie_same_origin);

// Cross-origin request contains cookies, only when sent in CORS mode, using
// crossOrigin = "use-credentials".
videoTest("cross-origin + undefined",
  cross_origin, '', undefined);
videoTest("cross-origin + anonymous",
  cross_origin, 'anonymous', undefined);
videoTest("cross-origin + use-credentials",
  cross_origin, 'use-credentials', cookie_cross_origin);
