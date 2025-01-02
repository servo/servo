precision mediump float;
uniform sampler2D s2D;
uniform samplerCube sCube;
void main()
{
    gl_FragColor = texture2D(s2D, vec2(0.5, 0.5)) +
                   textureCube(sCube, vec3(0.5, 0.5, 0.5));
}
