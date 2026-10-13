// META: title=Geometry of detached elements
// META: spec=https://drafts.csswg.org/cssom-view/

"use strict";

test(() => {

  const el = document.createElement("p");
  assert_equals(el.clientHeight, 0);

}, "clientHeight should return 0 for an element with no associated CSS layout box");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.clientLeft, 0);

}, "clientLeft should return 0 for an element with no associated CSS layout box");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.clientTop, 0);

}, "clientTop should return 0 for an element with no associated CSS layout box");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.clientWidth, 0);

}, "clientWidth should return 0 for an element with no associated CSS layout box");

test(() => {

  const zeroDimensions = {
    x: 0,
    y: 0,
    bottom: 0,
    height: 0,
    left: 0,
    right: 0,
    top: 0,
    width: 0
  };

  const el = document.createElement("p");
  const rect = el.getBoundingClientRect();
  for (const [property, value] of Object.entries(zeroDimensions)) {
    assert_equals(rect[property], value, property);
  }

}, "getBoundingClientRect should return 0 value dimensions for an element with no associated CSS layout box");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.scrollHeight, 0);

}, "scrollHeight should return 0 for an element disconnected from a document");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.scrollLeft, 0);

}, "scrollLeft should return 0 for an element disconnected from a document");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.scrollTop, 0);

}, "scrollTop should return 0 for an element disconnected from a document");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.scrollWidth, 0);

}, "scrollWidth should return 0 for an element disconnected from a document");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.offsetHeight, 0);

}, "offsetHeight should return 0 for an element disconnected from a document");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.offsetLeft, 0);

}, "offsetLeft should return 0 for an element disconnected from a document");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.offsetParent, null);

}, "offsetParent should return null for an element disconnected from a document");

test(() => {

  const el = document.createElement("p");
  assert_equals(el.offsetTop, 0);

}, "offsetTop should return 0 for an element disconnected from a document");
