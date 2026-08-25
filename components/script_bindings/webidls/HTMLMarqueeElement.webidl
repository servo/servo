/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-marquee-element
[Exposed=Window]
interface HTMLMarqueeElement : HTMLElement {
  [HTMLConstructor] constructor();

  // [CEReactions, Reflect] attribute DOMString behavior;
  // [CEReactions, Reflect] attribute DOMString bgColor;
  // [CEReactions, Reflect] attribute DOMString direction;
  // [CEReactions, Reflect] attribute DOMString height;
  // [CEReactions, Reflect] attribute unsigned long hspace;
  // [CEReactions] attribute long loop;
  // [CEReactions, Reflect, ReflectDefault=6] attribute unsigned long scrollAmount;
  // [CEReactions, Reflect, ReflectDefault=85] attribute unsigned long scrollDelay;
  // [CEReactions, Reflect] attribute boolean trueSpeed;
  // [CEReactions, Reflect] attribute unsigned long vspace;
  // [CEReactions, Reflect] attribute DOMString width;

  // undefined start();
  // undefined stop();
};
