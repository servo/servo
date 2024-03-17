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
  'condition-urlpattern-string-source-cache': [
    {condition: {urlPattern: '/**/cache.txt'}, source: 'cache'},
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
  'condition-invalid-request-method': [{
    condition: {requestMethod: String.fromCodePoint(0x3042)},
    source: 'network'
  }],
  'condition-invalid-or-condition-depth': (() => {
    const max = 10;
    const addOrCondition = (obj, depth) => {
      if (depth > max) {
        return obj;
      }
      return {
        urlPattern: `/foo-${depth}`,
        or: [addOrCondition(obj, depth + 1)]
      };
    };
    return {condition: addOrCondition({}, 0), source: 'network'};
  })(),
  'condition-invalid-router-size': [...Array(512)].map((val, i) => {
    return {
      condition: {urlPattern: `/foo-${i}`},
      source: 'network'
    };
  }),
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
};

export {routerRules};
