const TEST_CACHE_NAME = 'v1';

const routerRules = {
  'condition-urlpattern-constructed-source-network': [{
    condition: {urlPattern: new URLPattern({pathname: '/**/direct.txt'})},
    source: 'network'
  }],
  'condition-urlpattern-not-source-network': [{
    condition: {not: {urlPattern: new URLPattern({pathname: '/**/not.txt'})}},
    source: 'network'
  }],
  'condition-urlpattern-constructed-match-all-source-cache': [
    {condition: {urlPattern: new URLPattern({})}, source: 'cache'},
  ],
  'condition-urlpattern-urlpatterncompatible-source-network': [
    {condition: {urlPattern: {pathname: '/**/direct.txt'}}, source: 'network'},
  ],
  'condition-urlpattern-string-source-network': [
    {condition: {urlPattern: '/**/direct.txt'}, source: 'network'},
  ],
  'condition-urlpattern-string-source-cache': [
    {condition: {urlPattern: '/**/cache.txt'}, source: 'cache'},
  ],
  'condition-urlpattern-string-source-cache-with-name': [
    {condition: {urlPattern: '/**/cache.txt'}, source: {cacheName: TEST_CACHE_NAME}},
  ],
  'condition-urlpattern-constructed-ignore-case-source-network': [{
    condition: {
      urlPattern:
          new URLPattern({pathname: '/**/DiReCT.TxT'}, {ignoreCase: true})
    },
    source: 'network'
  }],
  'condition-urlpattern-constructed-respect-case-source-network': [{
    condition: {urlPattern: new URLPattern({pathname: '/**/DiReCT.TxT'})},
    source: 'network'
  }],
  'condition-request-source-network':
      [{condition: {requestMode: 'no-cors'}, source: 'network'}],
  'condition-request-navigate-source-cache':
      [{condition: {requestMode: 'navigate'}, source: 'cache'}],
  'condition-request-method-get-network':
      [{condition: {requestMethod: 'GET'}, source: 'network'}],
  'condition-request-method-post-network':
      [{condition: {requestMethod: 'POST'}, source: 'network'}],
  'condition-request-method-put-network':
      [{condition: {requestMethod: 'PUT'}, source: 'network'}],
  'condition-request-method-delete-network':
      [{condition: {requestMethod: 'DELETE'}, source: 'network'}],
  'condition-lack-of-condition': [{
    source: 'network'
  }],
  'condition-lack-of-source': [{
    condition: {requestMode: 'no-cors'},
  }],
  'condition-invalid-request-method': [{
    condition: {requestMethod: String.fromCodePoint(0x3042)},
    source: 'network'
  }],
  'condition-request-destination-script-network':
      [{condition: {requestDestination: 'script'}, source: 'network'}],
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
  'condition-request-source-fetch-event':
      [{condition: {requestMode: 'no-cors'}, source: 'fetch-event'}],
  'condition-urlpattern-string-source-fetch-event':
      [{condition: {urlPattern: '/**/*'}, source: 'fetch-event'}],
  'multiple-router-rules': [
    {
      condition: {
        urlPattern: '/**/direct.txt',
      },
      source: 'network'
    },
    {condition: {urlPattern: '/**/direct.html'}, source: 'network'}
  ],
  'condition-urlpattern-string-source-race-network-and-fetch-handler': [
    {
      condition: {urlPattern: '/**/direct.py'},
      source: 'race-network-and-fetch-handler'
    },
  ],
  'multiple-conditions-network': {
    condition: {
      urlPattern: new URLPattern({search: 'test'}),
      requestMode: 'cors',
      requestMethod: 'post',
    },
    source: 'network'
  },
  'multiple-conditions-with-destination-network' : {
    condition: {
      urlPattern: new URLPattern({search: 'test'}),
      requestDestination: 'style'
    },
    source: 'network'
  }
};

export {routerRules, TEST_CACHE_NAME as cacheName};
