/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-navigator-object
 * http://www.w3.org/TR/tracking-dnt/
 * http://www.w3.org/TR/geolocation-API/#geolocation_interface
 * http://www.w3.org/TR/battery-status/#navigatorbattery-interface
 * http://www.w3.org/TR/vibration/#vibration-interface
 * http://www.w3.org/2012/sysapps/runtime/#extension-to-the-navigator-interface-1
 * https://dvcs.w3.org/hg/gamepad/raw-file/default/gamepad.html#navigator-interface-extension
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface MozPowerManager;
interface MozWakeLock;

// http://www.whatwg.org/specs/web-apps/current-work/#the-navigator-object
[HeaderFile="Navigator.h", NeedNewResolve]
interface Navigator {
  // objects implementing this interface also implement the interfaces given below
};
Navigator implements NavigatorID;
Navigator implements NavigatorLanguage;
Navigator implements NavigatorOnLine;
//Navigator implements NavigatorContentUtils;
//Navigator implements NavigatorStorageUtils;

[NoInterfaceObject]
interface NavigatorID {
  readonly attribute DOMString appName;
  [Throws]
  readonly attribute DOMString appVersion;
  [Throws]
  readonly attribute DOMString platform;
  [Throws]
  readonly attribute DOMString userAgent;

  // Spec has this as a const, but that's wrong because it should not
  // be on the interface object.
  //const DOMString product = "Gecko"; // for historical reasons
  readonly attribute DOMString product;
};

[NoInterfaceObject]
interface NavigatorLanguage {
  readonly attribute DOMString? language;
};

[NoInterfaceObject]
interface NavigatorOnLine {
  readonly attribute boolean onLine;
};

/*
[NoInterfaceObject]
interface NavigatorContentUtils {
  // content handler registration
  [Throws]
  void registerProtocolHandler(DOMString scheme, DOMString url, DOMString title);
  [Throws]
  void registerContentHandler(DOMString mimeType, DOMString url, DOMString title);
  // NOT IMPLEMENTED
  //DOMString isProtocolHandlerRegistered(DOMString scheme, DOMString url);
  //DOMString isContentHandlerRegistered(DOMString mimeType, DOMString url);
  //void unregisterProtocolHandler(DOMString scheme, DOMString url);
  //void unregisterContentHandler(DOMString mimeType, DOMString url);
};

[NoInterfaceObject]
interface NavigatorStorageUtils {
  // NOT IMPLEMENTED
  //void yieldForStorageUpdates();
};

// Things that definitely need to be in the spec and and are not for some
// reason.  See https://www.w3.org/Bugs/Public/show_bug.cgi?id=22406
partial interface Navigator {
  [Throws]
  readonly attribute MimeTypeArray mimeTypes;
  [Throws]
  readonly attribute PluginArray plugins;
};
*/

// http://www.w3.org/TR/tracking-dnt/ sort of
partial interface Navigator {
  readonly attribute DOMString doNotTrack;
};

/*
// http://www.w3.org/TR/geolocation-API/#geolocation_interface
[NoInterfaceObject]
interface NavigatorGeolocation {
  [Throws, Pref="geo.enabled"]
  readonly attribute Geolocation geolocation;
};
Navigator implements NavigatorGeolocation;

// http://www.w3.org/TR/battery-status/#navigatorbattery-interface
[NoInterfaceObject]
interface NavigatorBattery {
    // XXXbz Per spec this should be non-nullable, but we return null in
    // torn-down windows.  See bug 884925.
    [Throws, Func="Navigator::HasBatterySupport"]
    readonly attribute BatteryManager? battery;
};
Navigator implements NavigatorBattery;
*/

/*
// http://www.w3.org/TR/vibration/#vibration-interface
partial interface Navigator {
    // We don't support sequences in unions yet
    //boolean vibrate ((unsigned long or sequence<unsigned long>) pattern);
    // XXXbz also, per spec we should be returning a boolean, and we just don't.
    // See bug 884935.
    [Throws]
    void vibrate(unsigned long duration);
    [Throws]
    void vibrate(sequence<unsigned long> pattern);
};
*/

// Mozilla-specific extensions

callback interface MozIdleObserver {
  // Time is in seconds and is read only when idle observers are added
  // and removed.
  readonly attribute unsigned long time;
  void onidle();
  void onactive();
};

// nsIDOMNavigator
partial interface Navigator {
  // WebKit/Blink/Trident/Presto support this (hardcoded "Mozilla").
  [Throws]
  readonly attribute DOMString appCodeName;
  //[Throws]
  //readonly attribute DOMString oscpu;
  // WebKit/Blink support this; Trident/Presto do not.
  readonly attribute DOMString vendor;
  // WebKit/Blink supports this (hardcoded ""); Trident/Presto do not.
  readonly attribute DOMString vendorSub;
  // WebKit/Blink supports this (hardcoded "20030107"); Trident/Presto don't
  readonly attribute DOMString productSub;
  // WebKit/Blink/Trident/Presto support this.
  readonly attribute boolean cookieEnabled;
  [Throws]
  readonly attribute DOMString buildID;
  //[Throws, Func="Navigator::HasPowerSupport"]
  //readonly attribute MozPowerManager mozPower;

  // WebKit/Blink/Trident/Presto support this.
  [Throws]
  boolean javaEnabled();
  // Everyone but WebKit/Blink supports this.  See bug 679971.
  boolean taintEnabled();

  /**
   * Navigator requests to add an idle observer to the existing window.
   */
  //[Throws, Func="Navigator::HasIdleSupport"]
  //void addIdleObserver(MozIdleObserver aIdleObserver);

  /**
   * Navigator requests to remove an idle observer from the existing window.
   */
  //[Throws, Func="Navigator::HasIdleSupport"]
  //void removeIdleObserver(MozIdleObserver aIdleObserver);

  /**
   * Request a wake lock for a resource.
   *
   * A page holds a wake lock to request that a resource not be turned
   * off (or otherwise made unavailable).
   *
   * The topic is the name of a resource that might be made unavailable for
   * various reasons. For example, on a mobile device the power manager might
   * decide to turn off the screen after a period of idle time to save power.
   *
   * The resource manager checks the lock state of a topic before turning off
   * the associated resource. For example, a page could hold a lock on the
   * "screen" topic to prevent the screensaver from appearing or the screen
   * from turning off.
   *
   * The resource manager defines what each topic means and sets policy.  For
   * example, the resource manager might decide to ignore 'screen' wake locks
   * held by pages which are not visible.
   *
   * One topic can be locked multiple times; it is considered released only when
   * all locks on the topic have been released.
   *
   * The returned nsIDOMMozWakeLock object is a token of the lock.  You can
   * unlock the lock via the object's |unlock| method.  The lock is released
   * automatically when its associated window is unloaded.
   *
   * @param aTopic resource name
   */
  //[Throws, Func="Navigator::HasWakeLockSupport"]
  //MozWakeLock requestWakeLock(DOMString aTopic);
};

/*
// nsIDOMNavigatorDeviceStorage
partial interface Navigator {
  [Throws, Pref="device.storage.enabled"]
  DeviceStorage? getDeviceStorage(DOMString type);
  [Throws, Pref="device.storage.enabled"]
  sequence<DeviceStorage> getDeviceStorages(DOMString type);
};

// nsIDOMNavigatorDesktopNotification
partial interface Navigator {
  [Throws, Func="Navigator::HasDesktopNotificationSupport"]
  readonly attribute DesktopNotificationCenter mozNotification;
};

// nsIDOMClientInformation
partial interface Navigator {
  [Throws]
  boolean mozIsLocallyAvailable(DOMString uri, boolean whenOffline);
};

// nsIDOMMozNavigatorMobileMessage
interface MozMobileMessageManager;
partial interface Navigator {
  [Func="Navigator::HasMobileMessageSupport"]
  readonly attribute MozMobileMessageManager? mozMobileMessage;
};

// nsIDOMMozNavigatorNetwork
interface MozConnection;
partial interface Navigator {
  [Pref="dom.network.enabled"]
  readonly attribute MozConnection? mozConnection;
};

// nsIDOMNavigatorCamera
partial interface Navigator {
  [Throws, Func="Navigator::HasCameraSupport"]
  readonly attribute CameraManager mozCameras;
};

// nsIDOMNavigatorSystemMessages and sort of maybe
// http://www.w3.org/2012/sysapps/runtime/#extension-to-the-navigator-interface-1
callback systemMessageCallback = void (optional object message);
partial interface Navigator {
  [Throws, Pref="dom.sysmsg.enabled"]
  void    mozSetMessageHandler (DOMString type, systemMessageCallback? callback);
  [Throws, Pref="dom.sysmsg.enabled"]
  boolean mozHasPendingMessage (DOMString type);
};

#ifdef MOZ_B2G_RIL
partial interface Navigator {
  [Throws, Func="Navigator::HasTelephonySupport"]
  readonly attribute Telephony? mozTelephony;
};

// nsIMozNavigatorMobileConnection
interface MozMobileConnection;
partial interface Navigator {
  [Throws, Func="Navigator::HasMobileConnectionSupport"]
  readonly attribute MozMobileConnection mozMobileConnection;
};

partial interface Navigator {
  [Throws, Func="Navigator::HasCellBroadcastSupport"]
  readonly attribute MozCellBroadcast mozCellBroadcast;
};

// nsIMozNavigatorVoicemail
interface MozVoicemail;
partial interface Navigator {
  [Throws, Func="Navigator::HasVoicemailSupport"]
  readonly attribute MozVoicemail mozVoicemail;
};

// nsIMozNavigatorIccManager
interface MozIccManager;
partial interface Navigator {
  [Throws, Func="Navigator::HasIccManagerSupport"]
  readonly attribute MozIccManager? mozIccManager;
};
#endif // MOZ_B2G_RIL

#ifdef MOZ_GAMEPAD
// https://dvcs.w3.org/hg/gamepad/raw-file/default/gamepad.html#navigator-interface-extension
partial interface Navigator {
  [Throws, Pref="dom.gamepad.enabled"]
  sequence<Gamepad?> getGamepads();
};
#endif // MOZ_GAMEPAD

#ifdef MOZ_B2G_BT
partial interface Navigator {
  [Throws, Func="Navigator::HasBluetoothSupport"]
  readonly attribute BluetoothManager mozBluetooth;
};
#endif // MOZ_B2G_BT

#ifdef MOZ_TIME_MANAGER
// nsIDOMMozNavigatorTime
partial interface Navigator {
  [Throws, Func="Navigator::HasTimeSupport"]
  readonly attribute MozTimeManager mozTime;
};
#endif // MOZ_TIME_MANAGER

#ifdef MOZ_AUDIO_CHANNEL_MANAGER
// nsIMozNavigatorAudioChannelManager
partial interface Navigator {
  [Throws]
  readonly attribute AudioChannelManager mozAudioChannelManager;
};
#endif // MOZ_AUDIO_CHANNEL_MANAGER

#ifdef MOZ_MEDIA_NAVIGATOR
// nsIDOMNavigatorUserMedia
callback MozDOMGetUserMediaSuccessCallback = void (nsISupports? value);
callback MozDOMGetUserMediaErrorCallback = void (DOMString error);
interface MozMediaStreamOptions;
partial interface Navigator {
  [Throws, Func="Navigator::HasUserMediaSupport"]
  void mozGetUserMedia(MozMediaStreamOptions? params,
                       MozDOMGetUserMediaSuccessCallback? onsuccess,
                       MozDOMGetUserMediaErrorCallback? onerror);
};

// nsINavigatorUserMedia
callback MozGetUserMediaDevicesSuccessCallback = void (nsIVariant? devices);
partial interface Navigator {
  [Throws, ChromeOnly]
  void mozGetUserMediaDevices(MozGetUserMediaDevicesSuccessCallback? onsuccess,
                              MozDOMGetUserMediaErrorCallback? onerror);
};
#endif // MOZ_MEDIA_NAVIGATOR
*/