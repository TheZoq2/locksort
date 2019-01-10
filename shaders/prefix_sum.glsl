#version 430

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(std430, binding = 3) buffer Input {
    float values[];
};

uniform uint iteration;

void prefix_sum() {
    uint offset = int(pow(2, iteration));

    uint base_index = gl_GlobalInvocationID.x;
    uint index = gl_GlobalInvocationID.x + offset;
    if (index < values.length()) {
        values[index] = values[base_index] + values[index];
    }
}

void main() {
    prefix_sum();
}
