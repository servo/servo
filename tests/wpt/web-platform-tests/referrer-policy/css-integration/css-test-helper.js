var svg_ns = "http://www.w3.org/2000/svg";
var url_prefix = location.protocol + "//" + location.hostname + ":" +
                 location.port + "/referrer-policy/generic/subresource/";

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

// Schedules async_test's for each of the test properties
// Parameters:
//     testProperties: An array of test properties.
//     testDescription: A test description
//     testFunction: A function call which sets up the expect result and runs
//                   the actual test
function runSvgTests(testProperties, testDescription, testFunction) {
  let runNextTest = function () {
    let property = testProperties.shift();
    if (property === undefined) {
      return;
    }

    let current = {
      test: async_test(testDescription + " " + property),
      id: token(),
      property: property,
    };

    testFunction(current);

    let check_url = url_prefix + "svg.py" + "?id=" + current.id +
                    "&report-headers";
    current.test.step_timeout(
      queryXhr.bind(this, check_url,
                    function(message) {
                      current.test.step(function() {
                        assert_own_property(message, "headers");
                        assert_own_property(message, "referrer");
                        assert_equals(message.referrer, current.expected);
                      });
                      current.test.done();
                    }),
      800);

  };

  add_result_callback(runNextTest);
  runNextTest();
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
