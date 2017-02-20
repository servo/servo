---
layout: page
title: Visual Tests
order: 7
---

Visual tests are typically used when testing rendering of things that
cannot be tested with [reftests][].

Their main advantage of over manual tests is they can be verified using
browser-specific and platform-specific screenshots; note, however, that many
browser vendors treat them identically to manual tests hence they are
similarly discouraged as they very infrequently, if ever, get run by them.

## Writing a Visual Test

Visuals tests are test files which have `-visual` at the end of their
filename, before the extension. There is nothing needed in them to
make them work.

They should follow the [general test guidelines][general guidelines],
especially noting the requirement to be self-describing (i.e., they
must give a clear pass condition in their rendering).

Similarly, they should consider the [rendering test guidelines][rendering],
especially those about color, to ensure those running the test don't
incorrectly judge its result.


[general guidelines]: {{ site.baseurl }}{% link _writing-tests/general-guidelines.md %}
[reftests]: {{ site.baseurl }}{% link _writing-tests/reftests.md %}
[rendering]: {{ site.baseurl }}{% link _writing-tests/rendering.md %}
