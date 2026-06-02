// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

class ContributeToHistogramOperation {
  async run(urls, data) {
    if (data.enableDebugMode) {
      privateAggregation.enableDebugMode(data.enableDebugModeArgs);

      if (data.enableDebugModeExtraTime) {
        privateAggregation.enableDebugMode(data.enableDebugModeArgs);
      }
    }
    for (const contribution of data.contributions) {
      privateAggregation.contributeToHistogram(contribution);
    }

    // If an error occurs, the default URL will be picked instead.
    return 1;
  }
}

register('contribute-to-histogram', ContributeToHistogramOperation);
