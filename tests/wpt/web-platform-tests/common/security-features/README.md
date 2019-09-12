This directory contains the common infrastructure for the following tests.
- referrer-policy/
- mixed-content/
- upgrade-insecure-requests/

Subdirectories:
- `subresource`:
    Serves subresources, with support for redirects, stash, etc.
    The subresource paths are managed by `subresourceMap` and
    fetched in `requestVia*()` functions in `resources/common.js`.
- `scope`:
    Serves nested contexts, such as iframe documents or workers.
    Used from `invokeFrom*()` functions in `resources/common.js`.

# spec.src.json format

## Source Contexts

In **`source_context_list_schema`**, we can specify

- source contexts from where subresource requests are sent, and
- how policies are delivered, by source contexts and/or subresource requests.

- `sourceContextList`: an array of `SourceContext` objects, and
- `subresourcePolicyDeliveries`: an array of `PolicyDelivery` objects.

They have the same object format as described in
`common/security-features/resources/common.js` comments, and are directly
serialized to generated HTML files and passed to JavaScript test code,
except that:

- The first entry of `sourceContextList`'s `sourceContextType` should be
  always `top`, which represents the top-level generated test HTML.
  (This entry is omitted in the JSON passed to JavaScript, but
  the policy deliveries specified here are written as e.g.
  <meta> elements in the generated test HTML or HTTP headers)
- Instead of `PolicyDelivery` object (in `sourceContextList` or
  `subresourcePolicyDeliveries`), following placeholder strings can be used.

The keys of `source_context_list_schema` can be used as the values of
`source_context_list` fields, to indicate which source context configuration
to be used.

## PolicyDelivery placeholders

Each test contains

- `delivery_key` (derived from the top-level `delivery_key`) and
- `delivery_value`, `delivery_type` (derived from `test_expansion`),

which represents the **target policy delivery**, the policy delivery to be
tested.

The following placeholder strings in `source_context_list_schema` can be used:

- `"policy"`:
    - Replaced with the target policy delivery.
    - Can be used to specify where the target policy delivery should be
      delivered.
- `"policyIfNonNull"`:
    - Replaced with the target policy delivery, only if it has non-null value.
      If the value is null, then the test file is not generated.
- `"anotherPolicy"`:
    - Replaced with a `PolicyDelivery` object that has a different value from
      the target policy delivery.
    - Can be used to specify e.g. a policy that should be overridden by
      the target policy delivery.

For example, when the target policy delivery is
{deliveryType: "http-rp", key: "referrerPolicy", value: "no-referrer"},

    "sourceContextList": [
      {"sourceContextType": "top", "policyDeliveries": ["anotherPolicy"]},
      {"sourceContextType": "classic-worker", "policyDeliveries": ["policy"]}
    ]

is replaced with

    "sourceContextList": [
      {"sourceContextType": "top", "policyDeliveries": [
          {"deliveryType": "meta",
           "key": "referrerPolicy",
           "value": "unsafe-url"}]
      },
      {"sourceContextType": "classic-worker", "policyDeliveries": [
          {"deliveryType": "http-rp",
           "key": "referrerPolicy",
           "value": "no-referrer"}]
      }
    ]

which indicates

- The top-level Document has `<meta name="referrer" content="unsafe-url">`.
- The classic worker is created with
  `Referrer-Policy: no-referrer` HTTP response headers.

## `source_context_schema` and `subresource_schema`

These represent supported delivery types and subresources
for each source context or subresource type. These are used

- To filter out test files for unsupported combinations of delivery types,
  source contexts and subresources.
- To determine what delivery types should be used for `anotherPolicy`
  placeholder.
