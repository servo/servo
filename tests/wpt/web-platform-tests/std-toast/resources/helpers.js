import { showToast, StdToastElement } from 'std:elements/toast';

// helper functions to keep tests from bleeding into each other

const runTest = (testFn, name, toast, action) => {
    try {
        test(() => {
            testFn(toast, action);
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

export const testActionToast = (testFn, name) => {
    const toast = new StdToastElement('Message', {});
    const action = document.createElement('button');
    action.setAttribute('slot', 'action');
    action.textContent = 'action';
    toast.appendChild(action);
    document.querySelector('main').appendChild(toast);

    runTest(testFn, name, toast, action);
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

export const assertActionButtonOnToast = (action, toast) => {
    assert_equals(toast.action, action);
    assert_equals(action.getAttribute('slot'), 'action');
    assert_equals(action, toast.querySelector('button'));
};

export const assertComputedStyleMapsEqual = (element1, element2) => {
    assert_greater_than(element1.computedStyleMap().size, 0);
    for (const [styleProperty, baseStyleValues] of element1.computedStyleMap()) {
        const refStyleValues = element2.computedStyleMap().getAll(styleProperty);
        assert_equals(baseStyleValues.length, refStyleValues.length, `${styleProperty} length`);

        for (let i = 0; i < baseStyleValues.length; ++i) {
            const baseStyleValue = baseStyleValues[i];
            const refStyleValue = refStyleValues[i];
            assert_equals(baseStyleValue.toString(), refStyleValue.toString(), `diff at value ${styleProperty}`);
        }
    }
}

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
