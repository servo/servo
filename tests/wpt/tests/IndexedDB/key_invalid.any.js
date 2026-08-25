// META: global=window,worker
// META: title=Invalid key
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#key-construct

'use strict';

const invalid_key = (desc, key) => {
  async_test(t => {
    const db = createdb_for_multiple_tests();
    let objStore = null;
    let objStore2 = null;

    const is_cloneable = o => {
      try {
        self.postMessage(o, '*');
        return true;
      } catch (ex) {
        return false;
      }
    };

    db.setTest(t).onupgradeneeded = t.step_func(e => {
      objStore = objStore || e.target.result.createObjectStore('store');
      assert_throws_dom('DataError', () => {
        objStore.add('value', key);
      });

      if (is_cloneable(key)) {
        objStore2 = objStore2 || e.target.result.createObjectStore('store2', {
          keyPath: ['x', 'keypath'],
        });
        assert_throws_dom('DataError', () => {
          objStore.add('value', key);
        });
      }
      t.done();
    });
  }, 'Invalid key - ' + desc);
};

const fake_array = {
  length: 0,
  constructor: Array,
};

const ArrayClone = function() {};
ArrayClone.prototype = Array;
const ArrayClone_instance = new ArrayClone();

// booleans
invalid_key('true', true);
invalid_key('false', false);

// null/NaN/undefined
invalid_key('null', null);
invalid_key('NaN', NaN);
invalid_key('undefined', undefined);
invalid_key('undefined2');

// functions
invalid_key('function() {}', function() {});

// objects
invalid_key('{}', {});
invalid_key('{ obj: 1 }', {obj: 1});
invalid_key('Math', Math);
invalid_key('self', self);
invalid_key('{length:0,constructor:Array}', fake_array);
invalid_key('Array clone’s instance', ArrayClone_instance);
invalid_key('Array (object)', Array);
invalid_key('String (object)', String);
invalid_key('new String()', new String());
invalid_key('new Number()', new Number());
invalid_key('new Boolean()', new Boolean());

// arrays
invalid_key('[{}]', [{}]);
invalid_key('[[], [], [], [[ Date ]]]', [[], [], [], [[Date]]]);
invalid_key('[undefined]', [undefined]);
invalid_key('[,1]', [, 1]);

if (typeof document !== 'undefined') {
  invalid_key(
      'document.getElementsByTagName("script")',
      document.getElementsByTagName('script'));
}

//  dates
invalid_key('new Date(NaN)', new Date(NaN));
invalid_key('new Date(Infinity)', new Date(Infinity));

// regexes
invalid_key('/foo/', /foo/);
invalid_key('new RegExp()', new RegExp());

const sparse = [];
sparse[10] = 'hei';
invalid_key('sparse array', sparse);

const sparse2 = [];
sparse2[0] = 1;
sparse2[''] = 2;
sparse2[2] = 3;
invalid_key('sparse array 2', sparse2);

invalid_key('[[1], [3], [7], [[ sparse array ]]]', [
  [1],
  [3],
  [7],
  [[sparse2]],
]);

// sparse3
invalid_key('[1,2,3,,]', [
  1,
  2,
  3,
  ,
]);

const recursive = [];
recursive.push(recursive);
invalid_key('array directly contains self', recursive);

const recursive2 = [];
recursive2.push([recursive2]);
invalid_key('array indirectly contains self', recursive2);

const recursive3 = [recursive];
invalid_key('array member contains self', recursive3);

invalid_key('proxy of an array', new Proxy([1, 2, 3], {}));
