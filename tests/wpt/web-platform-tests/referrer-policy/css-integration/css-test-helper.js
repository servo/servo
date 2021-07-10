var svg_ns = "http://www.w3.org/2000/svg";
var url_prefix = location.protocol + "//" + location.hostname + ":" +
                 location.port + "/common/security-features/subresource/";

var svg_test_properties = [
  'fill',
  'stroke',
  'filter',
  'clip-path',
  'marker-start',
  'marker-mid',
  'marker-end',
  'mask',
  'mask-image',
];

// Parameters:
//     testProperties: An array of test properties.
//     testDescription: A test description
//     testFunction: A function call which sets up the expect result and runs
//                   the actual test
function runSvgTests(testProperties, testDescription, testFunction) {
  for (const property of testProperties) {
    let current = {
      id: token(),
      property: property,
    };

    promise_test(t => {
      testFunction(current);
      return timeoutPromise(t, 800)
        .then(() => {
            let check_url = url_prefix + "svg.py" + "?id=" + current.id +
                            "&report-headers";
            return requestViaFetch(check_url);
          })
        .then(message => {
            assert_own_property(message, "headers");
            assert_own_property(message, "referrer");
            assert_equals(message.referrer, current.expected);
          });
      },
      testDescription + " " + property);
  }
}

function createSvg() {
  let svg = document.createElementNS(svg_ns, 'svg');
  svg.setAttribute('width', '400');
  svg.setAttribute('height', '400');
  let path = document.createElementNS(svg_ns, 'path');
  path.setAttribute('d', 'M 50,5 95,100 5,100 z');
  svg.appendChild(path);
  return svg;
}
