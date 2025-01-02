"use strict";

test(() => {
  assert_false('joinAdInterestGroup' in navigator, 'joinAdInterestGroup not available.');
  assert_false('leaveAdInterestGroup' in navigator, 'leaveAdInterestGroup not available.');
  assert_false('runAdAuction' in navigator, 'runAdAuction not available.');
  assert_false('updateAdInterestGroups' in navigator, 'updateAdInterestGroups not available.');
}, "Fledge requires secure context.");
