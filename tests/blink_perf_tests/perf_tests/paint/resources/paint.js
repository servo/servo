// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

function measurePaint(test) {
  test.tracingCategories = 'blink';
  test.traceEventsToMeasure = [
    'LocalFrameView::RunPrePaintLifecyclePhase',
    'LocalFrameView::RunPaintLifecyclePhase'
  ];
  PerfTestRunner.measureFrameTime(test);
}
