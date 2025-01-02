// META: title=XMLHttpRequest: open() call fires sync readystate event

const title = "XMLHttpRequest: open() call fires sync readystate event";

test(function () {
  var client = new XMLHttpRequest()
  var eventsFired = []
  client.onreadystatechange = function () {
    eventsFired.push(client.readyState)
  }
  client.open('GET', "...", false)
  assert_array_equals(eventsFired, [1])
}, title + ' (sync)');

test(function () {
  var client = new XMLHttpRequest()
  var eventsFired = []
  client.onreadystatechange = function () {
    eventsFired.push(client.readyState)
  }
  client.open('GET', "...", true)
  assert_array_equals(eventsFired, [1])
}, title + ' (async)');
