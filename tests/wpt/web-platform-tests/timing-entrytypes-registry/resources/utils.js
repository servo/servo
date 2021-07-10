const STEPS = {};

const types = (self.PerformanceObserver
                  && self.PerformanceObserver.supportedEntryTypes)?
    self.PerformanceObserver.supportedEntryTypes
    : undefined;

if (types) {
  // we observe everything as soon as possible
  new PerformanceObserver(function (list, observer) {
    for (const entry of list.getEntries())
      if (STEPS[entry.entryType]) STEPS[entry.entryType](entry);
  }).observe({entryTypes: self.PerformanceObserver.supportedEntryTypes});
}

function test_support(def) {
  if (!types || !types.includes(def[0])) {
    return;
  }
  const desc = `'${def[0]}' entries should be observable`;
  const t = async_test(desc);

  STEPS[def[0]] = (entry) => {
    t.step(() => assert_equals(Object.prototype.toString.call(entry),
    `[object ${def[1]}]`,
    `Class name of entry should be ${def[1]}.`));
    t.done();
  }
}
