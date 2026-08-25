// META: title=createImageBitmap resolves in a task
// META: script=/common/media.js
// META: script=./common.sub.js
function makeWorkerImageBitmap() {
    return makeOffscreenCanvas().then(canvas => {
        return createImageBitmap(canvas);
    });
}
var imageSourceTypes = self.GLOBAL.isWorker() ?
    [
        { name: 'an OffscreenCanvas', factory: makeOffscreenCanvas },
        { name: 'an ImageData', factory: makeImageData },
        { name: 'an ImageBitmap', factory: makeWorkerImageBitmap },
        { name: 'a Blob', factory: makeBlob('/images/pattern.png') },
    ] :
    imageSourceTypes;

var testFuncs = {
    reject_sync: (promiseFunc, source, t) => {
        return new Promise((resolve, reject) => {
            let rejected = false;
            promiseFunc(source).then(() => {
                reject(new Error('Expected this call to reject'));
            }, (err) => {
                rejected = true;
            });
            Promise.resolve().then(() => {
                try {
                    assert_equals(rejected, true, 'The promise should be rejected synchronously')
                    resolve(t);
                } catch (err) {
                    reject(err);
                }
            })
        });

    },
    resolve_async: (promiseFunc, source, t) => {
        return new Promise((resolve, reject) => {
            let taskRan = false;
            promiseFunc(source).then(() => {
                try {
                    assert_equals(taskRan, true, 'The promise should be resolved asynchronously')
                    resolve(t);
                } catch (err) {
                    reject(err)
                }
            }, t.unreached_func('Expected this call to resolve'))
            Promise.resolve().then(() => {
                taskRan = true;
            });
        });
    },
    reject_async: (promiseFunc, source, t) => {
        return new Promise((resolve, reject) => {
            let taskRan = false;
            promiseFunc(source).then(
                t.unreached_func('Expected this call to reject'),
                () => {
                    try {
                        assert_equals(taskRan, true, 'The promise should be rejected asynchronously')
                        resolve(t);
                    } catch (err) {
                        reject(err)
                    }
                },
            );
            Promise.resolve().then(() => {
                taskRan = true;
            });
        });
    },
};

var testCases = [{
        description: 'createImageBitmap with <sourceType> source and ' +
            'invalid cropHeight',
        promiseTestFunction: (source) => createImageBitmap(source, 0, 0, 0, 0),
        resolution: 'reject_sync'
    },
    {
        description: 'createImageBitmap with <sourceType> source and ' +
            'invalid resizeHeight',
        promiseTestFunction: (source) => createImageBitmap(source, {
            resizeWidth: 0,
            resizeHeight: 0
        }),
        resolution: 'reject_sync'
    },
    {
        description: 'createImageBitmap with <sourceType> source',
        promiseTestFunction: (source) => createImageBitmap(source),
        resolution: 'resolve_async'
    },
];

imageSourceTypes.forEach(imageSourceType => {
    testCases.forEach(testCase => {
        let description = `${testCase.description.replace('<sourceType>',
        imageSourceType.name)} should ${testCase.resolution.replace('_', ' ')}`;

        promise_test(t => {
            return imageSourceType.factory().then(source => {
                const tester = testFuncs[testCase.resolution];
                return tester(testCase.promiseTestFunction, source, t);
            })
        }, description);

    });
});
promise_test(t => {
    return makeBlob('data:,')().then(image => {
        return testFuncs.reject_async((source) => createImageBitmap(source), image, t);
    });
}, 'Invalid Blob source should reject async');