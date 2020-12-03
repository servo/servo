// META: global=window,worker
// META: script=resources/utils.js

test(() => {
  runTest({ pathname: '/foo/bar' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/ba' }, expected: false },
    { input: { pathname: '/foo/bar/' }, expected: false },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
  ]);
}, "fixed string");

test(() => {
  runTest({ pathname: '/foo/:bar' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/index.html' }, expected: true },
    { input: { pathname: '/foo/bar/' }, expected: false },
    { input: { pathname: '/foo/' }, expected: false },
  ]);
}, "named group");

test(() => {
  runTest({ pathname: '/foo/(.*)' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: true },
    { input: { pathname: '/foo/' }, expected: true },
    { input: { pathname: '/foo' }, expected: false },
  ]);
}, "regexp group");

test(() => {
  runTest({ pathname: '/foo/:bar(.*)' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: true },
    { input: { pathname: '/foo/' }, expected: true },
    { input: { pathname: '/foo' }, expected: false },
  ]);
}, "named regexp group");

test(() => {
  runTest({ pathname: '/foo/:bar?' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo' }, expected: true },
    { input: { pathname: '/foo/' }, expected: false },
    { input: { pathname: '/foobar' }, expected: false },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
  ]);
}, "optional named group");

test(() => {
  runTest({ pathname: '/foo/:bar+' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: true },
    { input: { pathname: '/foo' }, expected: false },
    { input: { pathname: '/foo/' }, expected: false },
    { input: { pathname: '/foobar' }, expected: false },
  ]);
}, "repeated named group");

test(() => {
  runTest({ pathname: '/foo/:bar*' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: true },
    { input: { pathname: '/foo' }, expected: true },
    { input: { pathname: '/foo/' }, expected: false },
    { input: { pathname: '/foobar' }, expected: false },
  ]);
}, "optional repeated named group");

test(() => {
  runTest({ pathname: '/foo/(.*)?' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: true },
    { input: { pathname: '/foo/' }, expected: true },
    { input: { pathname: '/foo' }, expected: true },
    { input: { pathname: '/fo' }, expected: false },
    { input: { pathname: '/foobar' }, expected: false },
  ]);
}, "optional regexp group");

test(() => {
  runTest({ pathname: '/foo/(.*)+' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: true },
    { input: { pathname: '/foo/' }, expected: true },
    { input: { pathname: '/foo' }, expected: false },
    { input: { pathname: '/fo' }, expected: false },
    { input: { pathname: '/foobar' }, expected: false },
  ]);
}, "repeated regexp group");

test(() => {
  runTest({ pathname: '/foo/(.*)*' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: true },
    { input: { pathname: '/foo/' }, expected: true },
    { input: { pathname: '/foo' }, expected: true },
    { input: { pathname: '/fo' }, expected: false },
    { input: { pathname: '/foobar' }, expected: false },
  ]);
}, "optional repeated regexp group");

test(() => {
  runTest({ pathname: '/foo{/bar}' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
    { input: { pathname: '/foo/' }, expected: false },
    { input: { pathname: '/foo' }, expected: false },
  ]);
}, "group");

test(() => {
  runTest({ pathname: '/foo{/bar}?' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
    { input: { pathname: '/foo' }, expected: true },
    { input: { pathname: '/foo/' }, expected: false },
  ]);
}, "optional group");

test(() => {
  runTest({ pathname: '/foo{/bar}+' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
    { input: { pathname: '/foo' }, expected: false },
    { input: { pathname: '/foo/' }, expected: false },
  ]);
}, "repeated group");

test(() => {
  runTest({ pathname: '/foo{/bar}*' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
    { input: { pathname: '/foo' }, expected: true },
    { input: { pathname: '/foo/' }, expected: false },
  ]);
}, "repeated optional group");
