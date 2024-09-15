// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: timeout=long

'use strict';

const TEST_VALUE = "popin-partition-test";

async function seedPartition(remoteContextWrapper, name) {
  await remoteContextWrapper.executeScript((name, TEST_VALUE) => {
    window.localStorage.setItem(name, TEST_VALUE);
    document.cookie = name + "-strict=" + TEST_VALUE + "; SameSite=Strict; Secure";
    document.cookie = name + "-lax=" + TEST_VALUE + "; SameSite=Lax; Secure";
    document.cookie = name + "-none=" + TEST_VALUE + "; SameSite=None; Secure";
    document.cookie = name + "-partitioned-strict=" + TEST_VALUE + "; Partitioned; SameSite=Strict; Secure";
    document.cookie = name + "-partitioned-lax=" + TEST_VALUE + "; Partitioned; SameSite=Lax; Secure";
    document.cookie = name + "-partitioned-none=" + TEST_VALUE + "; Partitioned; SameSite=None; Secure";
  }, [name, TEST_VALUE]);
}

async function openPopin(test, remoteContextWrapper, origin) {
  const popin = await remoteContextWrapper.addWindow(
    /*extraConfig=*/ origin ? { origin } : null,
    /*options=*/ { features: "popin" });
  assert_equals(await popin.executeScript(() => { return window.popinContextType(); }), "partitioned");
  test.add_cleanup(async () => {
    await popin.executeScript(() => { window.close(); });
  });
  return popin;
}

async function getCookies(remoteContextWrapper) {
  return await remoteContextWrapper.executeScript((TEST_VALUE) => {
    if (!document.cookie) {
      return [];
    }
    const cookies = document.cookie.split(";");
    let cookieNames = [];
    for (let i = 0; i < cookies.length; i++) {
      let cookieName = cookies[i].split("=")[0].trim();
      let cookieValue = cookies[i].split("=")[1].trim();
      if (cookieValue === TEST_VALUE) {
        cookieNames.push(cookieName);
      }
    }
    return cookieNames.sort();
  }, [TEST_VALUE]);
}

async function getLocalStorage(remoteContextWrapper) {
  return await remoteContextWrapper.executeScript((TEST_VALUE) => {
    let storageNames = [];
    for (let i = 0; i < window.localStorage.length; i++) {
      if (window.localStorage.getItem(window.localStorage.key(i)) === TEST_VALUE) {
        storageNames.push(window.localStorage.key(i));
      }
    }
    return storageNames.sort();
  }, [TEST_VALUE]);
}

const rcHelper = new RemoteContextHelper();
const handles = {};

promise_setup(async () => {
  assert_in_array("partitioned", window.popinContextTypesSupported());

  handles.main = await rcHelper.addWindow();
  handles.frameSameSite = await handles.main.addIframe();
  handles.frameCrossSite = await handles.main.addIframe(
    /*extraConfig=*/ {
      origin: "HTTPS_NOTSAMESITE_ORIGIN",
    },
    /*attributes=*/ {
      allow: "popins",
    }
  );
  handles.frameSameSiteWithCrossSiteAncestor =
    await handles.frameCrossSite.addIframe(
      /*extraConfig=*/ null,
      /*attributes=*/ {
        allow: "popins",
      }
    );
  handles.crossSite = await rcHelper.addWindow(
    /*extraConfig=*/ {
      origin: "HTTPS_NOTSAMESITE_ORIGIN",
    }
  );
  handles.crossSiteFrameSameSite = await handles.crossSite.addIframe();
  Object.freeze(handles);

  await seedPartition(handles.main, "main");
  await seedPartition(handles.frameSameSite, "frameSameSite");
  await seedPartition(handles.frameCrossSite, "frameCrossSite");
  await seedPartition(handles.frameSameSiteWithCrossSiteAncestor, "frameSameSiteWithCrossSiteAncestor");
  await seedPartition(handles.crossSite, "crossSite");
  await seedPartition(handles.crossSiteFrameSameSite, "crossSiteFrameSameSite");
});

promise_test(async t => {
  const popin = await openPopin(t, handles.main);

  // The popin, it's opener and all their ancestors are same-site, so the popin
  // should have access to all cookies and storage of the main host.
  assert_array_equals(
    await getCookies(popin),
    [
      "crossSiteFrameSameSite-none",
      "frameSameSite-lax",
      "frameSameSite-none",
      "frameSameSite-partitioned-lax",
      "frameSameSite-partitioned-none",
      "frameSameSite-partitioned-strict",
      "frameSameSite-strict",
      "frameSameSiteWithCrossSiteAncestor-none",
      "main-lax",
      "main-none",
      "main-partitioned-lax",
      "main-partitioned-none",
      "main-partitioned-strict",
      "main-strict",
    ]);
  assert_array_equals(await getLocalStorage(popin), ["frameSameSite", "main"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Main site opens same-site popin.");

promise_test(async t => {
  const popin = await openPopin(t, handles.main, "HTTPS_NOTSAMESITE_ORIGIN");

  // The popin is cross-site to its opener, so the popin should only have access
  // to the local storage variable and SameSite=None cookies of the cross-site
  // frame, and the unpartitioned SameSite=None cookie of the cross-site window.
  assert_array_equals(await getCookies(popin), ["crossSite-none", "frameCrossSite-none", "frameCrossSite-partitioned-none"]);
  assert_array_equals(await getLocalStorage(popin), ["frameCrossSite"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Main site opens cross-site popin.");

promise_test(async t => {
  const popin = await openPopin(t, handles.frameSameSite);

  // The popin, it's opener and all their ancestors are same-site, so the popin
  // should have access to all cookies and storage of the main host.
  assert_array_equals(
    await getCookies(popin),
    [
      "crossSiteFrameSameSite-none",
      "frameSameSite-lax",
      "frameSameSite-none",
      "frameSameSite-partitioned-lax",
      "frameSameSite-partitioned-none",
      "frameSameSite-partitioned-strict",
      "frameSameSite-strict",
      "frameSameSiteWithCrossSiteAncestor-none",
      "main-lax",
      "main-none",
      "main-partitioned-lax",
      "main-partitioned-none",
      "main-partitioned-strict",
      "main-strict",
    ]);
  assert_array_equals(await getLocalStorage(popin), ["frameSameSite", "main"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Same-site frame opens same-site popin.");

promise_test(async t => {
  const popin = await openPopin(t, handles.frameCrossSite);

  // The main-host popin has a cross-site ancestor, so it should only have
  // access to the local storage variable and SameSite=None cookies set in a
  // cross-site ancestor context, and the unpartitioned SameSite=None cookies of
  // the main-host contexts.
  assert_array_equals(await getCookies(popin),
    [
      "crossSiteFrameSameSite-none",
      "frameSameSite-none",
      "frameSameSiteWithCrossSiteAncestor-none",
      "frameSameSiteWithCrossSiteAncestor-partitioned-none",
      "main-none",
    ]);
  assert_array_equals(await getLocalStorage(popin), ["frameSameSiteWithCrossSiteAncestor"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Cross-site frame opens main-host popin.");

promise_test(async t => {
  const popin = await openPopin(t, handles.frameCrossSite, "HTTPS_NOTSAMESITE_ORIGIN");

  // The popin and its opener is cross-site to the main frame, so the popin
  // should only have access to the local storage variable and SameSite=None
  // cookies of the cross-site frame, and the unpartitioned SameSite=None cookie
  // of the cross-site window.
  assert_array_equals(await getCookies(popin), ["crossSite-none", "frameCrossSite-none", "frameCrossSite-partitioned-none"]);
  assert_array_equals(await getLocalStorage(popin), ["frameCrossSite"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Cross-site frame opens alternative-host popin.");

promise_test(async t => {
  const popin = await openPopin(t, handles.frameSameSiteWithCrossSiteAncestor);

  // The main-host popin has a cross-site ancestor, so it should only have
  // access to the local storage variable and SameSite=None cookies set in a
  // cross-site ancestor context, and the unpartitioned SameSite=None cookies of
  // the main-host contexts.
  assert_array_equals(await getCookies(popin),
    [
      "crossSiteFrameSameSite-none",
      "frameSameSite-none",
      "frameSameSiteWithCrossSiteAncestor-none",
      "frameSameSiteWithCrossSiteAncestor-partitioned-none",
      "main-none",
    ]);
  assert_array_equals(await getLocalStorage(popin), ["frameSameSiteWithCrossSiteAncestor"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Same-site frame with cross-site ancestor opens main-host popin.");

promise_test(async t => {
  const popin = await openPopin(t, handles.main);

  // The frame in the popin, the popin, it's opener and all their ancestors are
  // same-site, so the popin should have access to all cookies and storage of
  // the main host.
  const popinFrame = await popin.addIframe();
  assert_array_equals(
    await getCookies(popinFrame),
    [
      "crossSiteFrameSameSite-none",
      "frameSameSite-lax",
      "frameSameSite-none",
      "frameSameSite-partitioned-lax",
      "frameSameSite-partitioned-none",
      "frameSameSite-partitioned-strict",
      "frameSameSite-strict",
      "frameSameSiteWithCrossSiteAncestor-none",
      "main-lax",
      "main-none",
      "main-partitioned-lax",
      "main-partitioned-none",
      "main-partitioned-strict",
      "main-strict",
    ]);
  assert_array_equals(await getLocalStorage(popinFrame), ["frameSameSite", "main"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Main site opens same-site popin with same-site frame.");

promise_test(async t => {
  const popin = await openPopin(t, handles.main);

  const popinFrame = await popin.addIframe(
    /*extraConfig=*/ {
      origin: "HTTPS_NOTSAMESITE_ORIGIN",
    },
    /*attributes=*/ {
      allow: "popins",
    }
  );

  // The frame in the popin is cross-site to the main frame, so the popin should
  // only have access to the local storage variable and SameSite=None cookies of
  // the cross-site frame, and the unpartitioned SameSite=None cookie of the
  // cross-site window.
  assert_array_equals(await getCookies(popinFrame), ["crossSite-none", "frameCrossSite-none", "frameCrossSite-partitioned-none"]);
  assert_array_equals(await getLocalStorage(popinFrame), ["frameCrossSite"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Main site opens same-site popin with cross-site frame.");

promise_test(async t => {
  const popin = await openPopin(t, handles.main, "HTTPS_NOTSAMESITE_ORIGIN");

  const popinFrame = await popin.addIframe();

  // The main-host frame in the popin has a cross-site ancestor, so it should
  // only have access to the local storage variable and SameSite=None cookies
  // set in a cross-site ancestor context, and the unpartitioned SameSite=None
  // cookies of the main-host contexts.
  assert_array_equals(await getCookies(popinFrame),
    [
      "crossSiteFrameSameSite-none",
      "frameSameSite-none",
      "frameSameSiteWithCrossSiteAncestor-none",
      "frameSameSiteWithCrossSiteAncestor-partitioned-none",
      "main-none",
    ]);
  assert_array_equals(await getLocalStorage(popinFrame), ["frameSameSiteWithCrossSiteAncestor"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Main site opens cross-site popin with main-host frame.");

promise_test(async t => {
  const popin = await openPopin(t, handles.frameCrossSite);

  const popinFrame = await popin.addIframe(
    /*extraConfig=*/ {
      origin: "HTTPS_NOTSAMESITE_ORIGIN",
    },
    /*attributes=*/ {
      allow: "popins",
    }
  );

  // The frame in the popin is cross-site to the main frame, so the popin should
  // only have access to the local storage variable and SameSite=None cookies of
  // the cross-site frame, and the unpartitioned SameSite=Nonecookie of the
  // cross-site window.
  assert_array_equals(await getCookies(popinFrame), ["crossSite-none", "frameCrossSite-none", "frameCrossSite-partitioned-none"]);
  assert_array_equals(await getLocalStorage(popinFrame), ["frameCrossSite"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Cross-site frame opens main-host popin with cross-site frame.");

promise_test(async t => {
  const popin = await openPopin(t, handles.frameCrossSite, "HTTPS_NOTSAMESITE_ORIGIN");

  const popinFrame = await popin.addIframe();

  // The main-host frame in the popin has a cross-site ancestor, so it should
  // only have access to the local storage variable and SameSite=None cookies
  //set in a cross-site ancestor context, and the unpartitioned SameSite=None
  // cookies of the main-host contexts.
  assert_array_equals(await getCookies(popinFrame),
    [
      "crossSiteFrameSameSite-none",
      "frameSameSite-none",
      "frameSameSiteWithCrossSiteAncestor-none",
      "frameSameSiteWithCrossSiteAncestor-partitioned-none",
      "main-none",
    ]);
  assert_array_equals(await getLocalStorage(popinFrame), ["frameSameSiteWithCrossSiteAncestor"]);

  t.done();
}, "Verify Partitioned Popins have access to the proper cookie/storage partitions - Cross-site frame opens alternative-host popin with main-host frame.");
