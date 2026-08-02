#version 330 core

out vec4 FragColor;

in vec2 vTexCoord;
in vec4 vColor;

uniform sampler2D uTex;

void main() {
    float alpha = texture(uTex, vTexCoord).r;
    FragColor = vec4(vColor.rgb, alpha * vColor.a);
}