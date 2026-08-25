// META: title=Scheduler: Tasks run order when multiple scheduler interfere the same event loop
promise_test(async t => {

  const iframe = document.createElement("iframe");

  iframe.onload = async function() {
    const runOrder = [];
    const tasks = [];

    tasks.push(window.scheduler.postTask(function(){runOrder.push("outer")}));
    tasks.push(iframe.contentWindow.scheduler.postTask(function(){runOrder.push("inner")}));

    await Promise.all(tasks);

    assert_equals(runOrder.toString(),'outer,inner');
  }

  document.body.appendChild(iframe);
}, 'Test scheduler.postTask() from two different schedulers with the same priority will run based on their enqueue order.');

promise_test(async t => {
  const iframe = document.createElement("iframe");

  iframe.onload = async function() {
    const runOrder = [];
    const tasks = [];

    tasks.push(window.scheduler.postTask(function(){runOrder.push("outer")}));
    tasks.push(iframe.contentWindow.scheduler.postTask(function(){runOrder.push("inner")}, {priority: "user-blocking"}));

    await Promise.all(tasks);

    assert_equals(runOrder.toString(),'inner,outer');
  }

  document.body.appendChild(iframe);
}, 'Test scheduler.postTask() from two different schedulers with different priorities will run based on their priorities');
