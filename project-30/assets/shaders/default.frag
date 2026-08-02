#version 330 core

out vec4 FragColor;

in vec2 vTexCoord;
in vec4 vColor;

uniform sampler2D uTex;

void main() {
    FragColor = texture(uTex, vTexCoord) * vColor;
}