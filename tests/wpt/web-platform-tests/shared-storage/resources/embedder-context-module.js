// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

class ReportContext {
  async run(data) {
    if (!data || !data.hasOwnProperty('ancestorKey')) {
      return;
    }
    const ancestorKey = data['ancestorKey'];
    const context = sharedStorage.context;
    await sharedStorage.set(ancestorKey, context);
  }
}

register('report-context', ReportContext);
