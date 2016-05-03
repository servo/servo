var gEffectEasingTests = [
  {
    desc: 'steps(start) function',
    easing: 'steps(2, start)',
    easingFunction: stepStart(2)
  },
  {
    desc: 'steps(end) function',
    easing: 'steps(2, end)',
    easingFunction: stepEnd(2)
  },
  {
    desc: 'linear function',
    easing: 'linear', // cubic-bezier(0, 0, 1.0, 1.0)
    easingFunction: cubicBezier(0, 0, 1.0, 1.0)
  },
  {
    desc: 'ease function',
    easing: 'ease', // cubic-bezier(0.25, 0.1, 0.25, 1.0)
    easingFunction: cubicBezier(0.25, 0.1, 0.25, 1.0)
  },
  {
    desc: 'ease-in function',
    easing: 'ease-in', // cubic-bezier(0.42, 0, 1.0, 1.0)
    easingFunction: cubicBezier(0.42, 0, 1.0, 1.0)
  },
  {
    desc: 'ease-in-out function',
    easing: 'ease-in-out', // cubic-bezier(0.42, 0, 0.58, 1.0)
    easingFunction: cubicBezier(0.42, 0, 0.58, 1.0)
  },
  {
    desc: 'ease-out function',
    easing: 'ease-out', // cubic-bezier(0, 0, 0.58, 1.0)
    easingFunction: cubicBezier(0, 0, 0.58, 1.0)
  },
  {
    desc: 'easing function which produces values greater than 1',
    easing: 'cubic-bezier(0, 1.5, 1, 1.5)',
    easingFunction: cubicBezier(0, 1.5, 1, 1.5)
  }
];
