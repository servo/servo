'use strict';
function test_interpolation(settings, expectations, name) {
  var message_prefix = name ? name + ': ' : '';
  // Returns a timing function that at 0.5 evaluates to progress.
  function timingFunction(progress) {
    if (progress === 0)
      return 'steps(1, end)';
    if (progress === 1)
      return 'steps(1, start)';
    var y = (8 * progress - 1) / 6;
    return 'cubic-bezier(0, ' + y + ', 1, ' + y + ')';
  }

  function RoundMatrix(style) {
    var matrixMatch = style.match(/^(matrix(3d)?)\(.+\)$/);
    if (!!matrixMatch) {
      var matrixType = matrixMatch[1];
      var matrixArgs = style.substr(matrixType.length);
      var extractmatrix = function(matrixStr) {
        var list = [];
        var regex = /[+\-]?[0-9]+[.]?[0-9]*(e[+/-][0-9]+)?/g;
        var match = undefined;
        do {
          match = regex.exec(matrixStr);
          if (match) {
            list.push(parseFloat(parseFloat(match[0]).toFixed(6)));
          }
        } while (match);
        return list;
      }
      return matrixType + '(' + extractmatrix(matrixArgs).join(', ') + ')';
    }
    return style;
  }

  test(function(){
    assert_true(CSS.supports(settings.property, settings.from), 'Value "' + settings.from + '" is supported by ' + settings.property);
    assert_true(CSS.supports(settings.property, settings.to), 'Value "' + settings.to + '" is supported by ' + settings.property);
  }, message_prefix + '"' + settings.from + '" and "' + settings.to + '" are valid ' + settings.property + ' values');

  for (var i = 0; i < expectations.length; ++i) {
    var progress = expectations[i].at;
    var expectation = expectations[i].expect;
    var animationId = 'anim' + i;
    var targetId = 'target' + i;
    var referenceId = 'reference' + i;

    test(function(){
      assert_true(CSS.supports(settings.property, expectation), 'Value "' + expectation + '" is supported by ' + settings.property);

      var stylesheet = document.createElement('style');
      stylesheet.textContent =
        '#' + targetId + ' {\n' +
        '  animation: 2s ' + timingFunction(progress) + ' -1s paused ' + animationId + ';\n' +
        '}\n' +
        '@keyframes ' + animationId + ' {\n' +
        '  0% { ' + settings.property + ': ' + settings.from + '; }\n' +
        '  100% { ' + settings.property + ': ' + settings.to + '; }\n' +
        '}\n' +
        '#' + referenceId + ' {\n' +
        '  ' + settings.property + ': ' + expectation + ';\n' +
        '}\n';
      document.head.appendChild(stylesheet);

      var target = document.createElement('div');
      target.id = targetId;
      document.body.appendChild(target);

      var reference = document.createElement('div');
      reference.id = referenceId;
      document.body.appendChild(reference);
      reference.style = '';

      var observed = RoundMatrix(getComputedStyle(target)[settings.property]);
      var expected = RoundMatrix(getComputedStyle(reference)[settings.property]);
      assert_equals(observed, expected);
    }, message_prefix + 'Animation between "' + settings.from + '" and "' + settings.to + '" at progress ' + progress);
  }
}
