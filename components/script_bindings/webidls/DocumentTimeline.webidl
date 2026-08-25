/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/web-animations-1/#the-documenttimeline-interface
dictionary DocumentTimelineOptions {
  DOMHighResTimeStamp originTime = 0;
};

[Exposed=Window]
interface DocumentTimeline : AnimationTimeline {
  constructor(optional DocumentTimelineOptions options = {});
};
