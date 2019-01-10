#version 430

layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

layout(std430, binding = 3) buffer ToSort {
    vec4 values[];
};

uniform int current_block;
uniform int current_iteration;

void exchange(inout vec4 i, inout vec4 j) {
    vec4 k;
    k = i;
    i = j;
    j = k;
}

void bitonic_sort() {
    int index = int(gl_GlobalInvocationID.x);
    // The first chunk should be swaped up, the second down and so on
    int block_size = int(pow(2, current_block));
    int up = ((index / int(pow(2, current_block - 1))) % 2) * 2 - 1;

    // The step to take when swapping
    int step = int(pow(2, current_iteration));

    // The index to start swaping from
    int inner_block_size = int(pow(2, current_iteration)) * 2;
    int swap_index = (index / inner_block_size) * block_size + (index % inner_block_size);

    int inner_block_index = (index / step) * inner_block_size;
    int inner_block_offset = index % step;
    int final_index = inner_block_index + inner_block_offset;

    if(length(values[final_index]) * up > length(values[final_index + step]) * up) {
        exchange(values[final_index], values[final_index + step]);
    }
}

void main() {
    bitonic_sort();
}
