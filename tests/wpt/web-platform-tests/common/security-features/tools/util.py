import os, sys, json, json5, re
import collections

script_directory = os.path.dirname(os.path.abspath(__file__))
template_directory = os.path.abspath(
    os.path.join(script_directory, 'template'))
test_root_directory = os.path.abspath(
    os.path.join(script_directory, '..', '..', '..'))


def get_template(basename):
    with open(os.path.join(template_directory, basename), "r") as f:
        return f.read()


def write_file(filename, contents):
    with open(filename, "w") as f:
        f.write(contents)


def read_nth_line(fp, line_number):
    fp.seek(0)
    for i, line in enumerate(fp):
        if (i + 1) == line_number:
            return line


def load_spec_json(path_to_spec):
    re_error_location = re.compile('line ([0-9]+) column ([0-9]+)')
    with open(path_to_spec, "r") as f:
        try:
            return json5.load(f, object_pairs_hook=collections.OrderedDict)
        except ValueError as ex:
            print(ex.message)
            match = re_error_location.search(ex.message)
            if match:
                line_number, column = int(match.group(1)), int(match.group(2))
                print(read_nth_line(f, line_number).rstrip())
                print(" " * (column - 1) + "^")
            sys.exit(1)


class ShouldSkip(Exception):
    '''
    Raised when the given combination of subresource type, source context type,
    delivery type etc. are not supported and we should skip that configuration.
    ShouldSkip is expected in normal generator execution (and thus subsequent
    generation continues), as we first enumerate a broad range of configurations
    first, and later raise ShouldSkip to filter out unsupported combinations.

    ShouldSkip is distinguished from other general errors that cause immediate
    termination of the generator and require fix.
    '''
    def __init__(self):
        pass


class PolicyDelivery(object):
    '''
    See `@typedef PolicyDelivery` comments in
    `common/security-features/resources/common.sub.js`.
    '''

    def __init__(self, delivery_type, key, value):
        self.delivery_type = delivery_type
        self.key = key
        self.value = value

    def __eq__(self, other):
        return type(self) is type(other) and self.__dict__ == other.__dict__

    @classmethod
    def list_from_json(cls, list, target_policy_delivery,
                       supported_delivery_types):
        # type: (dict, PolicyDelivery, typing.List[str]) -> typing.List[PolicyDelivery]
        '''
        Parses a JSON object `list` that represents a list of `PolicyDelivery`
        and returns a list of `PolicyDelivery`, plus supporting placeholders
        (see `from_json()` comments below or
        `common/security-features/README.md`).

        Can raise `ShouldSkip`.
        '''
        if list is None:
            return []

        out = []
        for obj in list:
            policy_delivery = PolicyDelivery.from_json(
                obj, target_policy_delivery, supported_delivery_types)
            # Drop entries with null values.
            if policy_delivery.value is None:
                continue
            out.append(policy_delivery)
        return out

    @classmethod
    def from_json(cls, obj, target_policy_delivery, supported_delivery_types):
        # type: (dict, PolicyDelivery, typing.List[str]) -> PolicyDelivery
        '''
           Parses a JSON object `obj` and returns a `PolicyDelivery` object.
           In addition to dicts (in the same format as to_json() outputs),
           this method accepts the following placeholders:
             "policy":
               `target_policy_delivery`
             "policyIfNonNull":
               `target_policy_delivery` if its value is not None.
             "anotherPolicy":
               A PolicyDelivery that has the same key as
               `target_policy_delivery` but a different value.
               The delivery type is selected from `supported_delivery_types`.

        Can raise `ShouldSkip`.
        '''

        if obj == "policy":
            policy_delivery = target_policy_delivery
        elif obj == "nonNullPolicy":
            if target_policy_delivery.value is None:
                raise ShouldSkip()
            policy_delivery = target_policy_delivery
        elif obj == "anotherPolicy":
            if len(supported_delivery_types) == 0:
                raise ShouldSkip()
            policy_delivery = target_policy_delivery.get_another_policy(
                supported_delivery_types[0])
        elif isinstance(obj, dict):
            policy_delivery = PolicyDelivery(obj['deliveryType'], obj['key'],
                                             obj['value'])
        else:
            raise Exception('policy delivery is invalid: ' + obj)

        # Omit unsupported combinations of source contexts and delivery type.
        if policy_delivery.delivery_type not in supported_delivery_types:
            raise ShouldSkip()

        return policy_delivery

    def to_json(self):
        # type: () -> dict
        return {
            "deliveryType": self.delivery_type,
            "key": self.key,
            "value": self.value
        }

    def get_another_policy(self, delivery_type):
        # type: (str) -> PolicyDelivery
        if self.key == 'referrerPolicy':
            # Return 'unsafe-url' (i.e. more unsafe policy than `self.value`)
            # as long as possible, to make sure the tests to fail if the
            # returned policy is used unexpectedly instead of `self.value`.
            # Using safer policy wouldn't be distinguishable from acceptable
            # arbitrary policy enforcement by user agents, as specified at
            # Step 7 of
            # https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer:
            # "The user agent MAY alter referrerURL or referrerOrigin at this
            # point to enforce arbitrary policy considerations in the
            # interests of minimizing data leakage."
            # See also the comments at `referrerUrlResolver` in
            # `wpt/referrer-policy/generic/test-case.sub.js`.
            if self.value != 'unsafe-url':
                return PolicyDelivery(delivery_type, self.key, 'unsafe-url')
            else:
                return PolicyDelivery(delivery_type, self.key, 'no-referrer')
        elif self.key == 'mixedContent':
            if self.value == 'opt-in':
                return PolicyDelivery(delivery_type, self.key, None)
            else:
                return PolicyDelivery(delivery_type, self.key, 'opt-in')
        elif self.key == 'contentSecurityPolicy':
            if self.value is not None:
                return PolicyDelivery(delivery_type, self.key, None)
            else:
                return PolicyDelivery(delivery_type, self.key, 'worker-src-none')
        elif self.key == 'upgradeInsecureRequests':
            if self.value == 'upgrade':
                return PolicyDelivery(delivery_type, self.key, None)
            else:
                return PolicyDelivery(delivery_type, self.key, 'upgrade')
        else:
            raise Exception('delivery key is invalid: ' + self.key)


class SourceContext(object):
    def __init__(self, source_context_type, policy_deliveries):
        # type: (unicode, typing.List[PolicyDelivery]) -> None
        self.source_context_type = source_context_type
        self.policy_deliveries = policy_deliveries

    def __eq__(self, other):
        return type(self) is type(other) and self.__dict__ == other.__dict__

    @classmethod
    def from_json(cls, obj, target_policy_delivery, source_context_schema):
        '''
        Parses a JSON object `obj` and returns a `SourceContext` object.

        `target_policy_delivery` and `source_context_schema` are used for
        policy delivery placeholders and filtering out unsupported
        delivery types.

        Can raise `ShouldSkip`.
        '''
        source_context_type = obj.get('sourceContextType')
        policy_deliveries = PolicyDelivery.list_from_json(
            obj.get('policyDeliveries'), target_policy_delivery,
            source_context_schema['supported_delivery_type']
            [source_context_type])
        return SourceContext(source_context_type, policy_deliveries)

    def to_json(self):
        return {
            "sourceContextType": self.source_context_type,
            "policyDeliveries": [x.to_json() for x in self.policy_deliveries]
        }


class CustomEncoder(json.JSONEncoder):
    '''
    Used to dump dicts containing `SourceContext`/`PolicyDelivery` into JSON.
    '''
    def default(self, obj):
        if isinstance(obj, SourceContext):
            return obj.to_json()
        if isinstance(obj, PolicyDelivery):
            return obj.to_json()
        return json.JSONEncoder.default(self, obj)
