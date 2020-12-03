// META: global=window,worker
// META: script=resources/utils.js

// This file attempts to test the different ways you can pass input values
// to be matched; e.g. as strings vs structured component parts.

test(() => {
  runTest({ pathname: '/foo/bar' }, [
    { input: "https://example.com/foo/bar", expected: true },
    { input: "https://example.com/foo/bar/baz", expected: false },
  ]);
}, "init single component, input string");

test(() => {
  runTest({ pathname: '/foo/bar' }, [
    { input: { pathname: '/foo/bar' }, expected: true },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
  ]);
}, "init single component, input single component");

test(() => {
  runTest({ pathname: '/foo/bar' }, [
    { input: { hostname: 'example.com', pathname: '/foo/bar' },
      expected: true },
    { input: { hostname: 'example.com', pathname: '/foo/bar/baz' },
      expected: false },
  ]);
}, "init single component, input two components");

test(() => {
  runTest({ pathname: '/foo/bar' }, [
    { input: { pathname: '/foo/bar', baseURL: 'https://example.com' },
      expected: true },
    { input: { pathname: '/foo/bar/baz', baseURL: 'https://example.com' },
      expected: false },
  ]);
}, "init single component, input baseURL and single component");

test(() => {
  runTest({ pathname: '/foo/bar', baseURL: 'https://example.com?query#hash' }, [
    { input: "https://example.com/foo/bar", expected: true },
    { input: "https://example.com/foo/bar/baz", expected: false },
    { input: "https://example2.com/foo/bar", expected: false },
    { input: "http://example.com/foo/bar", expected: false },
  ]);
}, "init baseURL and single component, input string");

test(() => {
  runTest({ pathname: '/foo/bar', baseURL: 'https://example.com?query#hash' }, [
    { input: { pathname: '/foo/bar' }, expected: false },
    { input: { pathname: '/foo/bar/baz' }, expected: false },
  ]);
}, "init baseURL and single component, input single component");

test(() => {
  runTest({ pathname: '/foo/bar', baseURL: 'https://example.com?query#hash' }, [
    { input: { hostname: 'example.com', pathname: '/foo/bar' },
      expected: false },
    { input: { hostname: 'example.com', pathname: '/foo/bar/baz' },
      expected: false },
    { input: { hostname: 'example2.com', pathname: '/foo/bar' },
      expected: false },
  ]);
}, "init baseURL and single component, input two components");

test(() => {
  runTest({ pathname: '/foo/bar', baseURL: 'https://example.com?query#hash' }, [
    { input: { protocol: 'https', hostname: 'example.com',
               pathname: '/foo/bar' },
      expected: true },
    { input: { protocol: 'https', hostname: 'example.com',
               pathname: '/foo/bar/baz' },
      expected: false },
  ]);
}, "init single component, input three components");

test(() => {
  runTest({ pathname: '/foo/bar', baseURL: 'https://example.com?query#hash' }, [
    { input: { pathname: '/foo/bar', baseURL: 'https://example.com' },
      expected: true },
    { input: { pathname: '/foo/bar/baz', baseURL: 'https://example.com' },
      expected: false },
    { input: { pathname: '/foo/bar', baseURL: 'https://example2.com' },
      expected: false },
    { input: { pathname: '/foo/bar', baseURL: 'http://example.com' },
      expected: false },
  ]);
}, "init baseURL and single component, input baseURL and single component");
