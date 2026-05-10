// META: script=/common/gc.js
// META: title=Regression test for https://crbug.com/472771920
'use strict';

promise_test(async () => {
  let observable = new Observable(subscriber => {
    subscriber.next(1);
  });

  const promise = observable.toArray();
  observable = null;
  await garbageCollect();
}, "toArray(): does not crash on observable being GCed");

promise_test(async () => {
  let observable = new Observable(subscriber => {
    subscriber.next(1);
  });

  const promise = observable.forEach(() => {});
  observable = null;
  await garbageCollect();
}, "forEach(): does not crash on observable being GCed");

promise_test(async () => {
  let observable = new Observable(subscriber => {});

  const promise = observable.first();
  observable = null;
  await garbageCollect();
}, "first(): does not crash on observable being GCed");

promise_test(async () => {
  let observable = new Observable(subscriber => {});

  const promise = observable.last();
  observable = null;
  await garbageCollect();
}, "last(): does not crash on observable being GCed");

promise_test(async () => {
  let observable = new Observable(subscriber => {});

  const promise = observable.some(value => true);
  observable = null;
  await garbageCollect();
}, "some(): does not crash on observable being GCed");

promise_test(async () => {
  let observable = new Observable(subscriber => {});

  const promise = observable.every(value => true);
  observable = null;
  await garbageCollect();
}, "every(): does not crash on observable being GCed");

promise_test(async () => {
  let observable = new Observable(subscriber => {});

  const promise = observable.find(value => true);
  observable = null;
  await garbageCollect();
}, "find(): does not crash on observable being GCed");

promise_test(async () => {
  let observable = new Observable(subscriber => {});

  const promise = observable.reduce(value => value);
  observable = null;
  await garbageCollect();
}, "reduce(): does not crash on observable being GCed");

