/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void main(void)
{
#ifdef SERVO_ES2
    float alpha = Texture(sDiffuse, vColorTexCoord.xy).a;
#else
    float alpha = Texture(sDiffuse, vColorTexCoord.xy).r;
#endif
    SetFragColor(vec4(vColor.xyz, vColor.w * alpha));
}
