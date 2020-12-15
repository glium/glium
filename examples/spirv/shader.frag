// convert to SPIR-V with: glslangValidator -G shader.frag -o frag.spv

#version 430

layout(location = 0) out vec4 color;

void main() {
    vec2 q = gl_FragCoord.xy / 1024.;
    color = vec4(q, 1., 1.);
}
