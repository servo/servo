uniform int ival;
uniform int ival2[2];

void main()
{
    int sum = ival + ival2[0] + ival2[1];
    gl_Position = vec4(sum, 0.0, 0.0, 1.0);
}
