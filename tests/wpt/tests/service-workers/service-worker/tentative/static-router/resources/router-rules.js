// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
const routerRules = {
  'condition-urlpattern-constructed-source-network': [{
    condition: {urlPattern: new URLPattern({pathname: '/**/direct.txt'})},
    source: 'network'
  }],
  'condition-urlpattern-urlpatterninit-source-network': [
    {condition: {urlPattern: {pathname: '/**/direct.txt'}}, source: 'network'},
  ],
  'condition-urlpattern-string-source-network': [
    {condition: {urlPattern: '/**/direct.txt'}, source: 'network'},
  ],
  'condition-request-source-network':
      [{condition: {requestMode: 'no-cors'}, source: 'network'}],
  'condition-or-source-network': [{
    condition: {
      or: [
        {
          or: [{urlPattern: '/**/or-test/direct1.*??*'}],
        },
        {urlPattern: '/**/or-test/direct2.*??*'}
      ]
    },
    source: 'network'
  }],
};

export {routerRules};
