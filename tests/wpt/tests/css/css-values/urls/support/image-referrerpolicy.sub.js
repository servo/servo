function make_test_url(type, policy, expected) {
  if (type == "cross-origin")
    return `url("http://{{hosts[][]}}:{{ports[http][1]}}/css/css-values/urls/support/image-referrerpolicy.py?expected_referrer=${expected}&origin=${location.origin}/&url=${document.URL}"${policy ? ` referrerpolicy(${policy})` : ``})`;
  if (type == "same-origin")
    return `url("http://{{hosts[][]}}:{{ports[http][0]}}/css/css-values/urls/support/image-referrerpolicy.py?expected_referrer=${expected}&origin=${location.origin}/&url=${document.URL}"${policy ? ` referrerpolicy(${policy})` : ``})`;
  throw `Unknown type: ${type}`;
}

function test_image_referrer_policy(descriptor) {
  var style = document.createElement("style");
  style.innerHTML = `
  .test {
    width: 200px;
    height: 200px;
    background-color: blue;
    background-image: ${make_test_url(descriptor.load_type, descriptor.referrer_policy, descriptor.expected_referrer)};
  };`;
  document.head.append(style);
}
