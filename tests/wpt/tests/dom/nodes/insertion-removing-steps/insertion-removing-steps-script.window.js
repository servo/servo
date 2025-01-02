promise_test(async t => {
  const fragmentWithTwoScripts = new DocumentFragment();
  const script0 = document.createElement('script');
  const script1 = fragmentWithTwoScripts.appendChild(document.createElement('script'));
  const script2 = fragmentWithTwoScripts.appendChild(document.createElement('script'));

  window.kBaselineNumberOfScripts = document.scripts.length;
  assert_equals(document.scripts.length, kBaselineNumberOfScripts,
      "The WPT infra starts out with exactly 3 scripts");

  window.script0Executed = false;
  script0.innerText = `
    script0Executed = true;
    assert_equals(document.scripts.length, kBaselineNumberOfScripts + 1,
        'script0 can observe itself and no other scripts');
  `;

  window.script1Executed = false;
  script1.innerText = `
    script1Executed = true;
    assert_equals(document.scripts.length, kBaselineNumberOfScripts + 2,
        "script1 executes synchronously, and thus observes only itself and " +
        "previous scripts");
  `;

  window.script2Executed = false;
  script2.innerText = `
    script2Executed = true;
    assert_equals(document.scripts.length, kBaselineNumberOfScripts + 3,
        "script2 executes synchronously, and thus observes itself and all " +
        "previous scripts");
  `;

  assert_false(script0Executed, "Script0 does not execute before append()");
  document.body.append(script0);
  assert_true(script0Executed,
      "Script0 executes synchronously during append()");

  assert_false(script1Executed, "Script1 does not execute before append()");
  assert_false(script2Executed, "Script2 does not execute before append()");
  document.body.append(fragmentWithTwoScripts);
  assert_true(script1Executed,
      "Script1 executes synchronously during fragment append()");
  assert_true(script2Executed,
      "Script2 executes synchronously during fragment append()");
}, "Script node insertion is not atomic with regard to execution. Each " +
   "script is synchronously executed during the HTML element insertion " +
   "steps hook");
