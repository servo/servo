import { showToast, StdToastElement } from 'std:elements/toast';

// helper functions to keep tests from bleeding into each other

const runTest = (testFn, name, toast) => {
    try {
        test(() => {
            testFn(toast);
        }, name);
    } finally {
        toast.remove();
    }
};

const runTestAsync = (testFn, name, toast) => {
    async_test(t => {
        testFn(t, toast);
        t.add_cleanup(() => {
            toast.remove();
        });
    }, name);
};

export const testToastElement = (testFn, name) => {
    const toast = new StdToastElement('Message', {});
    document.querySelector('main').appendChild(toast);

    runTest(testFn, name, toast);
};

export const testToastElementAsync = (testFn, name) => {
    const toast = new StdToastElement('Message', {});
    document.querySelector('main').appendChild(toast);

    runTestAsync(testFn, name, toast);
};

export const testShowToast = (testFn, name) => {
    const toast = showToast("message");

    runTest(testFn, name, toast);
};

export const assertToastShown = (toast) => {
    assert_not_equals(window.getComputedStyle(toast).display, 'none');
    assert_true(toast.hasAttribute('open'));
    assert_true(toast.open);
};

export const assertToastNotShown = (toast) => {
    assert_equals(window.getComputedStyle(toast).display, 'none');
    assert_false(toast.hasAttribute('open'));
    assert_false(toast.open);
};

export class EventCollector {
    events = [];

    getCallback() {
        return (e) => {this.events.push(e)};
    }

    getCount() {
        return this.events.length;
    }

    getEvents() {
        return this.events;
    }
}
