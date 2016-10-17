/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/tests#test-runner

// callback BluetoothManualChooserEventsCallback = void(sequence<DOMString> events);

[Pref="dom.bluetooth.testing.enabled", Exposed=Window]
interface TestRunner {
  [Throws]
  void setBluetoothMockDataSet(DOMString dataSetName);
  // void setBluetoothManualChooser();
  // void getBluetoothManualChooserEvents(BluetoothManualChooserEventsCallback callback);
  // void sendBluetoothManualChooserEvent(DOMString event, DOMString argument);
};
