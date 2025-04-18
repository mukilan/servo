/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_BLUETOOTH

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothuuid

[Exposed=Window, Pref="dom_bluetooth_enabled"]
interface BluetoothUUID {
  [Throws]
  static UUID getService(BluetoothServiceUUID name);
  [Throws]
  static UUID getCharacteristic(BluetoothCharacteristicUUID name);
  [Throws]
  static UUID getDescriptor(BluetoothDescriptorUUID name);
  static UUID canonicalUUID([EnforceRange] unsigned long alias);
};

typedef DOMString UUID;
typedef (DOMString or unsigned long) BluetoothServiceUUID;
typedef (DOMString or unsigned long) BluetoothCharacteristicUUID;
typedef (DOMString or unsigned long) BluetoothDescriptorUUID;
