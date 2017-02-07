const discreteType = {
  testInterpolation: function(property, setup, options) {
    options.forEach(function(keyframes) {
      var [ from, to ] = keyframes;
      test(function(t) {
        var idlName = propertyToIDL(property);
        var target = createTestElement(t, setup);
        var animation = target.animate({ [idlName]: [from, to] },
                                       { duration: 1000, fill: 'both' });
        testAnimationSamples(animation, idlName,
                             [{ time: 0,    expected: from.toLowerCase() },
                              { time: 499,  expected: from.toLowerCase() },
                              { time: 500,  expected: to.toLowerCase() },
                              { time: 1000, expected: to.toLowerCase() }]);
      }, property + ' uses discrete animation when animating between "'
         + from + '" and "' + to + '" with linear easing');

      test(function(t) {
        // Easing: http://cubic-bezier.com/#.68,0,1,.01
        // With this curve, we don't reach the 50% point until about 95% of
        // the time has expired.
        var idlName = propertyToIDL(property);
        var keyframes = {};
        var target = createTestElement(t, setup);
        var animation = target.animate({ [idlName]: [from, to] },
                                       { duration: 1000, fill: 'both',
                                         easing: 'cubic-bezier(0.68,0,1,0.01)' });
        testAnimationSamples(animation, idlName,
                             [{ time: 0,    expected: from.toLowerCase() },
                              { time: 940,  expected: from.toLowerCase() },
                              { time: 960,  expected: to.toLowerCase() }]);
      }, property + ' uses discrete animation when animating between "'
         + from + '" and "' + to + '" with effect easing');

      test(function(t) {
        // Easing: http://cubic-bezier.com/#.68,0,1,.01
        // With this curve, we don't reach the 50% point until about 95% of
        // the time has expired.
        var idlName = propertyToIDL(property);
        var target = createTestElement(t, setup);
        var animation = target.animate({ [idlName]: [from, to],
                                         easing: 'cubic-bezier(0.68,0,1,0.01)' },
                                       { duration: 1000, fill: 'both' });
        testAnimationSamples(animation, idlName,
                             [{ time: 0,    expected: from.toLowerCase() },
                              { time: 940,  expected: from.toLowerCase() },
                              { time: 960,  expected: to.toLowerCase() }]);
      }, property + ' uses discrete animation when animating between "'
         + from + '" and "' + to + '" with keyframe easing');
    });
  },

  testAddition: function(property, setup, options) {
    options.forEach(function(keyframes) {
      var [ from, to ] = keyframes;
      test(function(t) {
        var idlName = propertyToIDL(property);
        var target = createTestElement(t, setup);
        target.animate({ [idlName]: [from, from] }, 1000);
        var animation = target.animate({ [idlName]: [to, to] },
                                       { duration: 1000, composite: 'add' });
        testAnimationSamples(animation, idlName,
                             [{ time: 0, expected: to.toLowerCase() }]);
      }, property + ': "' + to + '" onto "' + from + '"');

      test(function(t) {
        var idlName = propertyToIDL(property);
        var target = createTestElement(t, setup);
        target.animate({ [idlName]: [to, to] }, 1000);
        var animation = target.animate({ [idlName]: [from, from] },
                                       { duration: 1000, composite: 'add' });
        testAnimationSamples(animation, idlName,
                             [{ time: 0, expected: from.toLowerCase() }]);
      }, property + ': "' + from + '" onto "' + to + '"');
    });
  },

};

const lengthType = {
  testInterpolation: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['10px', '50px'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '10px' },
                            { time: 500,  expected: '30px' },
                            { time: 1000, expected: '50px' }]);
    }, property + ' supports animating as a length');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['1rem', '5rem'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '10px' },
                            { time: 500,  expected: '30px' },
                            { time: 1000, expected: '50px' }]);
    }, property + ' supports animating as a length of rem');
  },

  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '10px';
      var animation = target.animate({ [idlName]: ['10px', '50px'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName, [{ time: 0, expected: '20px' }]);
    }, property + ': length');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '1rem';
      var animation = target.animate({ [idlName]: ['1rem', '5rem'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName, [{ time: 0, expected: '20px' }]);
    }, property + ': length of rem');
  },

};

const percentageType = {
  testInterpolation: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['10%', '50%'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '10%' },
                            { time: 500,  expected: '30%' },
                            { time: 1000, expected: '50%' }]);
    }, property + ' supports animating as a percentage');
  },

  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '60%';
      var animation = target.animate({ [idlName]: ['70%', '100%'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName, [{ time: 0, expected: '130%' }]);
    }, property + ': percentage');
  },

};

const integerType = {
  testInterpolation: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: [-2, 2] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '-2' },
                            { time: 500,  expected: '0' },
                            { time: 1000, expected: '2' }]);
    }, property + ' supports animating as an integer');
  },

  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = -1;
      var animation = target.animate({ [idlName]: [-2, 2] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '-3' }]);
    }, property + ': integer');
  },

};

const lengthPercentageOrCalcType = {
  testInterpolation: function(property, setup) {
    lengthType.testInterpolation(property, setup);
    percentageType.testInterpolation(property, setup);

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['10px', '20%'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '10px' },
                            { time: 500,  expected: 'calc(5px + 10%)' },
                            { time: 1000, expected: '20%' }]);
    }, property + ' supports animating as combination units "px" and "%"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['10%', '2em'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '10%' },
                            { time: 500,  expected: 'calc(10px + 5%)' },
                            { time: 1000, expected: '20px' }]);
    }, property + ' supports animating as combination units "%" and "em"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['1em', '2rem'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '10px' },
                            { time: 500,  expected: '15px' },
                            { time: 1000, expected: '20px' }]);
    }, property + ' supports animating as combination units "em" and "rem"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['10px', 'calc(1em + 20%)'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '10px' },
                            { time: 500,  expected: 'calc(10px + 10%)' },
                            { time: 1000, expected: 'calc(10px + 20%)' }]);
    }, property + ' supports animating as combination units "px" and "calc"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate(
        { [idlName]: ['calc(10px + 10%)', 'calc(1em + 1rem + 20%)'] },
        { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,
                              expected: 'calc(10px + 10%)' },
                            { time: 500,
                              expected: 'calc(15px + 15%)' },
                            { time: 1000,
                              expected: 'calc(20px + 20%)' }]);
    }, property + ' supports animating as a calc');
  },

  testAddition: function(property, setup) {
    lengthType.testAddition(property, setup);
    percentageType.testAddition(property, setup);

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '10px';
      var animation = target.animate({ [idlName]: ['10%', '50%'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0, expected: 'calc(10px + 10%)' }]);
    }, property + ': units "%" onto "px"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '10%';
      var animation = target.animate({ [idlName]: ['10px', '50px'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0, expected: 'calc(10px + 10%)' }]);
    }, property + ': units "px" onto "%"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '10%';
      var animation = target.animate({ [idlName]: ['2rem', '5rem'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0, expected: 'calc(20px + 10%)' }]);
    }, property + ': units "rem" onto "%"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '2rem';
      var animation = target.animate({ [idlName]: ['10%', '50%'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0, expected: 'calc(20px + 10%)' }]);
    }, property + ': units "%" onto "rem"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '2em';
      var animation = target.animate({ [idlName]: ['2rem', '5rem'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName, [{ time: 0, expected: '40px' }]);
    }, property + ': units "rem" onto "em"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '2rem';
      var animation = target.animate({ [idlName]: ['2em', '5em'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName, [{ time: 0, expected: '40px' }]);
    }, property + ': units "em" onto "rem"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = '10px';
      var animation = target.animate({ [idlName]: ['calc(2em + 20%)',
                                                   'calc(5rem + 50%)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0, expected: 'calc(30px + 20%)' }]);
    }, property + ': units "calc" onto "px"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'calc(10px + 10%)';
      var animation = target.animate({ [idlName]: ['calc(20px + 20%)',
                                                   'calc(2em + 3rem + 40%)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0, expected: 'calc(30px + 30%)' }]);
    }, property + ': calc');
  },

};

const positiveNumberType = {
  testInterpolation: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: [1.1, 1.5] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: '1.1' },
                            { time: 500,  expected: '1.3' },
                            { time: 1000, expected: '1.5' }]);
    }, property + ' supports animating as a positive number');
  },

  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 1.1;
      var animation = target.animate({ [idlName]: [1.1, 1.5] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName, [{ time: 0, expected: '2.2' }]);
    }, property + ': positive number');
  },

};

const visibilityType = {
  testInterpolation: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['visible', 'hidden'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: 'visible' },
                            { time: 999,  expected: 'visible' },
                            { time: 1000, expected: 'hidden' }]);
    }, property + ' uses visibility animation when animating '
       + 'from "visible" to "hidden"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['hidden', 'visible'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: 'hidden' },
                            { time: 1,    expected: 'visible' },
                            { time: 1000, expected: 'visible' }]);
    }, property + ' uses visibility animation when animating '
     + 'from "hidden" to "visible"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['hidden', 'collapse'] },
                                     { duration: 1000, fill: 'both' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: 'hidden' },
                            { time: 499,  expected: 'hidden' },
                            { time: 500,  expected: 'collapse' },
                            { time: 1000, expected: 'collapse' }]);
    }, property + ' uses visibility animation when animating '
     + 'from "hidden" to "collapse"');

    test(function(t) {
      // Easing: http://cubic-bezier.com/#.68,-.55,.26,1.55
      // With this curve, the value is less than 0 till about 34%
      // also more than 1 since about 63%
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation =
        target.animate({ [idlName]: ['visible', 'hidden'] },
                       { duration: 1000, fill: 'both',
                         easing: 'cubic-bezier(0.68, -0.55, 0.26, 1.55)' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: 'visible' },
                            { time: 1,    expected: 'visible' },
                            { time: 330,  expected: 'visible' },
                            { time: 340,  expected: 'visible' },
                            { time: 620,  expected: 'visible' },
                            { time: 630,  expected: 'hidden' },
                            { time: 1000, expected: 'hidden' }]);
    }, property + ' uses visibility animation when animating '
     + 'from "visible" to "hidden" with easeInOutBack easing');
  },

  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'visible';
      var animation = target.animate({ [idlName]: ['visible', 'hidden'] },
                                     { duration: 1000, fill: 'both',
                                       composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: 'visible' },
                            { time: 1000, expected: 'visible' }]);
    }, property + ': onto "visible"');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'hidden';
      var animation = target.animate({ [idlName]: ['hidden', 'visible'] },
                                     { duration: 1000, fill: 'both',
                                       composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,    expected: 'hidden' },
                            { time: 1000, expected: 'visible' }]);
    }, property + ': onto "hidden"');
  },

};

const colorType = {
  testInterpolation: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['rgb(255, 0, 0)',
                                                   'rgb(0, 0, 255)'] },
                                     1000);
      testAnimationSamples(animation, idlName,
                           [{ time: 500,  expected: 'rgb(128, 0, 128)' }]);
    }, property + ' supports animating as color of rgb()');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['#ff0000', '#0000ff'] },
                                     1000);
      testAnimationSamples(animation, idlName,
                           [{ time: 500,  expected: 'rgb(128, 0, 128)' }]);
    }, property + ' supports animating as color of #RGB');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['hsl(0,   100%, 50%)',
                                                   'hsl(240, 100%, 50%)'] },
                                     1000);
      testAnimationSamples(animation, idlName,
                           [{ time: 500,  expected: 'rgb(128, 0, 128)' }]);
    }, property + ' supports animating as color of hsl()');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['#ff000066', '#0000ffcc'] },
                                     1000);
                                             // R: 255 * (0.4 * 0.5) / 0.6 = 85
                                             // G: 255 * (0.8 * 0.5) / 0.6 = 170
      testAnimationSamples(animation, idlName,
                           [{ time: 500,  expected: 'rgba(85, 0, 170, 0.6)' }]);
    }, property + ' supports animating as color of #RGBa');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['rgba(255, 0, 0, 0.4)',
                                                   'rgba(0, 0, 255, 0.8)'] },
                                     1000);
      testAnimationSamples(animation, idlName,      // Same as above.
                           [{ time: 500,  expected: 'rgba(85, 0, 170, 0.6)' }]);
    }, property + ' supports animating as color of rgba()');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['hsla(0,   100%, 50%, 0.4)',
                                                   'hsla(240, 100%, 50%, 0.8)'] },
                                     1000);
      testAnimationSamples(animation, idlName,      // Same as above.
                           [{ time: 500,  expected: 'rgba(85, 0, 170, 0.6)' }]);
    }, property + ' supports animating as color of hsla()');
  },

  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(128, 128, 128)';
      var animation = target.animate({ [idlName]: ['rgb(255, 0, 0)',
                                                   'rgb(0, 0, 255)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,   expected: 'rgb(255, 128, 128)' },
                            // The value at 50% is interpolated
                            // from 'rgb(128+255, 128, 128)'
                            // to   'rgb(128,     128, 128+255)'.
                            { time: 500, expected: 'rgb(255, 128, 255)' }]);
    }, property + ' supports animating as color of rgb() with overflowed ' +
       'from and to values');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(128, 128, 128)';
      var animation = target.animate({ [idlName]: ['#ff0000', '#0000ff'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,  expected: 'rgb(255, 128, 128)' }]);
    }, property + ' supports animating as color of #RGB');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(128, 128, 128)';
      var animation = target.animate({ [idlName]: ['hsl(0,   100%, 50%)',
                                                   'hsl(240, 100%, 50%)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,  expected: 'rgb(255, 128, 128)' }]);
    }, property + ' supports animating as color of hsl()');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(128, 128, 128)';
      var animation = target.animate({ [idlName]: ['#ff000066', '#0000ffcc'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
                           [{ time: 0,  expected: 'rgb(230, 128, 128)' }]);
    }, property + ' supports animating as color of #RGBa');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(128, 128, 128)';
      var animation = target.animate({ [idlName]: ['rgba(255, 0, 0, 0.4)',
                                                   'rgba(0, 0, 255, 0.8)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,      // Same as above.
                           [{ time: 0,  expected: 'rgb(230, 128, 128)' }]);
    }, property + ' supports animating as color of rgba()');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(128, 128, 128)';
      var animation = target.animate({ [idlName]: ['hsla(0,   100%, 50%, 0.4)',
                                                   'hsla(240, 100%, 50%, 0.8)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,      // Same as above.
                           [{ time: 0,  expected: 'rgb(230, 128, 128)' }]);
    }, property + ' supports animating as color of hsla()');
  },

};

const transformListType = {
  testInterpolation: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['translate(200px, -200px)',
                                                   'translate(400px, 400px)'] },
                                     1000);
      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500,  expected: [ 1, 0, 0, 1, 300, 100 ] }]);
    }, property + ': translate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['rotate(45deg)',
                                                   'rotate(135deg)'] },
                                     1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500,  expected: [ Math.cos(Math.PI / 2),
                                   Math.sin(Math.PI / 2),
                                  -Math.sin(Math.PI / 2),
                                   Math.cos(Math.PI / 2),
                                   0, 0] }]);
    }, property + ': rotate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['scale(3)', 'scale(5)'] },
                                     1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500,  expected: [ 4, 0, 0, 4, 0, 0 ] }]);
    }, property + ': scale');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation = target.animate({ [idlName]: ['skew(30deg, 60deg)',
                                                   'skew(60deg, 30deg)'] },
                                     1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500,  expected: [ 1, Math.tan(Math.PI / 4),
                                   Math.tan(Math.PI / 4), 1,
                                   0, 0] }]);
    }, property + ': skew');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation =
        target.animate({ [idlName]: ['translateX(100px) rotate(45deg)',
                                     'translateX(200px) rotate(135deg)'] },
                       1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500,  expected: [ Math.cos(Math.PI / 2),
                                   Math.sin(Math.PI / 2),
                                  -Math.sin(Math.PI / 2),
                                   Math.cos(Math.PI / 2),
                                   150, 0 ] }]);
    }, property + ': rotate and translate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation =
        target.animate({ [idlName]: ['rotate(45deg) translateX(100px)',
                                     'rotate(135deg) translateX(200px)'] },
                       1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500, expected: [ Math.cos(Math.PI / 2),
                                  Math.sin(Math.PI / 2),
                                 -Math.sin(Math.PI / 2),
                                  Math.cos(Math.PI / 2),
                                  150 * Math.cos(Math.PI / 2),
                                  150 * Math.sin(Math.PI / 2) ] }]);
    }, property + ': translate and rotate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation =                // matrix(0, 1, -1, 0, 0, 100)
        target.animate({ [idlName]: ['rotate(90deg) translateX(100px)',
                                     // matrix(-1, 0, 0, -1, 200, 0)
                                     'translateX(200px) rotate(180deg)'] },
                       1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500, expected: [ Math.cos(Math.PI * 3 / 4),
                                  Math.sin(Math.PI * 3 / 4),
                                 -Math.sin(Math.PI * 3 / 4),
                                  Math.cos(Math.PI * 3 / 4),
                                  100, 50 ] }]);
    }, property + ': mismatch order of translate and rotate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation =                 // Same matrices as above.
        target.animate({ [idlName]: [ 'matrix(0, 1, -1, 0, 0, 100)',
                                      'matrix(-1, 0, 0, -1, 200, 0)' ] },
                       1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500, expected: [ Math.cos(Math.PI * 3 / 4),
                                  Math.sin(Math.PI * 3 / 4),
                                 -Math.sin(Math.PI * 3 / 4),
                                  Math.cos(Math.PI * 3 / 4),
                                  100, 50 ] }]);
    }, property + ': matrix');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      var animation =
        target.animate({ [idlName]: [ 'rotate3d(1, 1, 0, 0deg)',
                                      'rotate3d(1, 1, 0, 90deg)'] },
                       1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500, expected: rotate3dToMatrix(1, 1, 0, Math.PI / 4) }]);
    }, property + ': rotate3d');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      // To calculate expected matrices easily, generate input matrices from
      // rotate3d.
      var from = rotate3dToMatrix3d(1, 1, 0, Math.PI / 4);
      var to = rotate3dToMatrix3d(1, 1, 0, Math.PI * 3 / 4);
      var animation =
        target.animate({ [idlName]: [ from, to ] }, 1000);

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 500, expected: rotate3dToMatrix(1, 1, 0, Math.PI * 2 / 4) }]);
    }, property + ': matrix3d');

  },

  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'translateX(100px)';
      var animation = target.animate({ [idlName]: ['translateX(-200px)',
                                                   'translateX(500px)'] },
                                     { duration: 1000, fill: 'both',
                                       composite: 'add' });
      testAnimationSampleMatrices(animation, idlName,
        [ { time: 0,    expected: [ 1, 0, 0, 1, -100, 0 ] },
          { time: 1000, expected: [ 1, 0, 0, 1,  600, 0 ] }]);
    }, property + ': translate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rotate(45deg)';
      var animation = target.animate({ [idlName]: ['rotate(-90deg)',
                                                   'rotate(90deg)'] },
                                     { duration: 1000, fill: 'both',
                                       composite: 'add' });

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: [ Math.cos(-Math.PI / 4),
                                   Math.sin(-Math.PI / 4),
                                  -Math.sin(-Math.PI / 4),
                                   Math.cos(-Math.PI / 4),
                                   0, 0] },
         { time: 1000, expected: [ Math.cos(Math.PI * 3 / 4),
                                   Math.sin(Math.PI * 3 / 4),
                                  -Math.sin(Math.PI * 3 / 4),
                                   Math.cos(Math.PI * 3 / 4),
                                   0, 0] }]);
    }, property + ': rotate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'scale(2)';
      var animation = target.animate({ [idlName]: ['scale(-3)', 'scale(5)'] },
                                     { duration: 1000, fill: 'both',
                                       composite: 'add' });

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: [ -6, 0, 0, -6, 0, 0 ] }, // scale(-3) scale(2)
         { time: 1000, expected: [ 10, 0, 0, 10, 0, 0 ] }]); // scale(5) scale(2)
    }, property + ': scale');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
                              // matrix(1, tan(10deg), tan(10deg), 1)
      target.style[idlName] = 'skew(10deg, 10deg)';
      var animation =                // matrix(1, tan(20deg), tan(-30deg), 1)
        target.animate({ [idlName]: ['skew(-30deg, 20deg)',
                                     // matrix(1, tan(-30deg), tan(20deg), 1)
                                     'skew(20deg, -30deg)'] },
                       { duration: 1000, fill: 'both', composite: 'add' });

      // matrix at 0%.
      // [ 1          tan(10deg) ] [ 1          tan(-30deg) ]
      // [ tan(10deg)          1 ] [ tan(20deg)           1 ] =
      //
      // [ 1 + tan(10deg) * tan(20deg) tan(-30deg) + tan(10deg)     ]
      // [     tan(10deg) + tan(20deg) tan(10deg) * tan(-30deg) + 1 ]

      // matrix at 100%.
      // [ 1          tan(10deg) ] [ 1           tan(20deg) ]
      // [ tan(10deg)          1 ] [ tan(-30deg)          1 ] =
      //
      // [ 1 + tan(10deg) * tan(-30deg) tan(20deg) + tan(10deg)     ]
      // [     tan(10deg) + tan(-30deg) tan(10deg) * tan(20deg) + 1 ]

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: [ 1 + Math.tan(Math.PI/18) * Math.tan(Math.PI/9),
                                   Math.tan(Math.PI/18) + Math.tan(Math.PI/9),
                                   Math.tan(-Math.PI/6) + Math.tan(Math.PI/18),
                                   1 + Math.tan(Math.PI/18) * Math.tan(-Math.PI/6),
                                   0, 0] },
         { time: 1000, expected: [ 1 + Math.tan(Math.PI/18) * Math.tan(-Math.PI/6),
                                   Math.tan(Math.PI/18) + Math.tan(-Math.PI/6),
                                   Math.tan(Math.PI/9) + Math.tan(Math.PI/18),
                                   1 + Math.tan(Math.PI/18) * Math.tan(Math.PI/9),
                                   0, 0] }]);
    }, property + ': skew');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
                               // matrix(1, 0, 0, 1, 100, 0)
      target.style[idlName] = 'translateX(100px)';
      var animation =                // matrix(0, 1, -1, 0, 0, 0)
        target.animate({ [idlName]: ['rotate(90deg)',
                                     // matrix(-1, 0, 0, -1, 0, 0)
                                     'rotate(180deg)'] },
                       { duration: 1000, fill: 'both', composite: 'add' });

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: [  0, 1, -1,  0, 100, 0 ] },
         { time: 1000, expected: [ -1, 0,  0, -1, 100, 0 ] }]);
    }, property + ': rotate on translate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
                               // matrix(0, 1, -1, 0, 0, 0)
      target.style[idlName] = 'rotate(90deg)';
      var animation =                // matrix(1, 0, 0, 1, 100, 0)
        target.animate({ [idlName]: ['translateX(100px)',
                                     // matrix(1, 0, 0, 1, 200, 0)
                                     'translateX(200px)'] },
                       { duration: 1000, fill: 'both', composite: 'add' });

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: [ 0, 1, -1, 0, 0, 100 ] },
         { time: 1000, expected: [ 0, 1, -1, 0, 0, 200 ] }]);
    }, property + ': translate on rotate');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'matrix(0, 1, -1, 0, 0, 0)';
      var animation =                 // Same matrices as above.
        target.animate({ [idlName]: [ 'matrix(1, 0, 0, 1, 100, 0)',
                                      'matrix(1, 0, 0, 1, 200, 0)' ] },
                       { duration: 1000, fill: 'both', composite: 'add' });

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: [ 0, 1, -1, 0, 0, 100 ] },
         { time: 1000, expected: [ 0, 1, -1, 0, 0, 200 ] }]);
    }, property + ': matrix');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rotate3d(1, 1, 0, 45deg)';
      var animation =
        target.animate({ [idlName]: [ 'rotate3d(1, 1, 0, -90deg)',
                                      'rotate3d(1, 1, 0, 90deg)'] },
                       { duration: 1000, fill: 'both', composite: 'add' });

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: rotate3dToMatrix(1, 1, 0,    -Math.PI / 4) },
         { time: 1000, expected: rotate3dToMatrix(1, 1, 0, 3 * Math.PI / 4) }]);
    }, property + ': rotate3d');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      // To calculate expected matrices easily, generate input matrices from
      // rotate3d.
      target.style[idlName] = rotate3dToMatrix3d(1, 1, 0, Math.PI / 4);
      var from = rotate3dToMatrix3d(1, 1, 0, -Math.PI / 2);
      var to = rotate3dToMatrix3d(1, 1, 0, Math.PI / 2);
      var animation =
        target.animate({ [idlName]: [ from, to ] },
                       { duration: 1000, fill: 'both', composite: 'add' });

      testAnimationSampleMatrices(animation, idlName,
        [{ time: 0,    expected: rotate3dToMatrix(1, 1, 0,    -Math.PI / 4) },
         { time: 1000, expected: rotate3dToMatrix(1, 1, 0, 3 * Math.PI / 4) }]);
    }, property + ': matrix3d');
  },

};

const filterListType = {
  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'blur(10px)';
      var animation = target.animate({ [idlName]: ['blur(20px)',
                                                   'blur(50px)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
        [ { time: 0,    expected: 'blur(10px) blur(20px)' }]);
    }, property + ': blur on blur');

    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'blur(10px)';
      var animation = target.animate({ [idlName]: ['brightness(80%)',
                                                   'brightness(40%)'] },
                                     { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
        [ { time: 0,    expected: 'blur(10px) brightness(0.8)' }]);
    }, property + ': different filter functions');
  },
};

const textShadowListType = {
  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(0, 0, 0) 0px 0px 0px';
      var animation =
        target.animate({ [idlName]: [ 'rgb(120, 120, 120) 10px 10px 10px',
                                      'rgb(120, 120, 120) 10px 10px 10px'] },
                       { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
        [ { time: 0, expected: 'rgb(0, 0, 0) 0px 0px 0px, ' +
                               'rgb(120, 120, 120) 10px 10px 10px' }]);
    }, property + ': shadow');
  },
};


const boxShadowListType = {
  testAddition: function(property, setup) {
    test(function(t) {
      var idlName = propertyToIDL(property);
      var target = createTestElement(t, setup);
      target.style[idlName] = 'rgb(0, 0, 0) 0px 0px 0px 0px';
      var animation =
        target.animate({ [idlName]: [ 'rgb(120, 120, 120) 10px 10px 10px 0px',
                                      'rgb(120, 120, 120) 10px 10px 10px 0px'] },
                       { duration: 1000, composite: 'add' });
      testAnimationSamples(animation, idlName,
        [ { time: 0, expected: 'rgb(0, 0, 0) 0px 0px 0px 0px, ' +
                               'rgb(120, 120, 120) 10px 10px 10px 0px' }]);
    }, property + ': shadow');
  },
};

const types = {
  color: colorType,
  discrete: discreteType,
  filterList: filterListType,
  integer: integerType,
  length: lengthType,
  percentage: percentageType,
  lengthPercentageOrCalc: lengthPercentageOrCalcType,
  positiveNumber: positiveNumberType,
  transformList: transformListType,
  visibility: visibilityType,
  boxShadowList: boxShadowListType,
  textShadowList: textShadowListType,
};

