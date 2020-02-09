// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

const WebContactsTest = (() => {
  class MockContacts {
    constructor() {
      this.bindingSet_ = new mojo.BindingSet(blink.mojom.ContactsManager);

      this.interceptor_ = new MojoInterfaceInterceptor(
        blink.mojom.ContactsManager.name, "context", true);
      this.interceptor_.oninterfacerequest =
          e => this.bindingSet_.addBinding(this, e.handle);
      this.interceptor_.start();

      this.selectedContacts_ = [];
    }

    formatAddress_(address) {
      // These are all required fields in the mojo definition.
      return {
        country: address.country || '',
        addressLine: address.addressLine || [],
        region: address.region || '',
        city: address.city || '',
        dependentLocality: address.dependentLocality || '',
        postalCode: address.postCode || '',
        sortingCode: address.sortingCode || '',
        organization: address.organization || '',
        recipient: address.recipient || '',
        phone: address.phone || '',
      };
    }

    async select(multiple, includeNames, includeEmails, includeTel, includeAddresses, includeIcons) {
      if (this.selectedContacts_ === null)
        return {contacts: null};

      const contactInfos = await Promise.all(this.selectedContacts_.map(async contact => {
        const contactInfo = new blink.mojom.ContactInfo();
        if (includeNames)
          contactInfo.name = contact.name || [];
        if (includeEmails)
          contactInfo.email = contact.email || [];
        if (includeTel)
          contactInfo.tel = contact.tel || [];
        if (includeAddresses) {
          contactInfo.address = (contact.address || []).map(address => this.formatAddress_(address));
        }
        if (includeIcons) {
          contactInfo.icon = await Promise.all(
            (contact.icon || []).map(async blob => ({
              mimeType: blob.type,
              data: (await blob.text()).split('').map(s => s.charCodeAt(0)),
            })));
        }
        return contactInfo;
      }));

      if (!contactInfos.length) return {contacts: []};
      if (!multiple) return {contacts: [contactInfos[0]]};
      return {contacts: contactInfos};
    }

    setSelectedContacts(contacts) {
      this.selectedContacts_ = contacts;
    }

    reset() {
      this.bindingSet_.closeAllBindings();
      this.interceptor_.stop();
    }
  }

  const mockContacts = new MockContacts();

  class ContactsTestChromium {
    constructor() {
      Object.freeze(this); // Make it immutable.
    }

    setSelectedContacts(contacts) {
      mockContacts.setSelectedContacts(contacts);
    }
  }

  return ContactsTestChromium;
})();
