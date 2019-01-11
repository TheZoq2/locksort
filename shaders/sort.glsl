#version 430

layout(local_size_x = 1, local_size_y = 256, local_size_z = 1) in;

layout(std430, binding = 3) buffer ToSort {
    vec4 values[];
};

uniform int current_block;
uniform int current_iteration;
uniform uint width;
uniform uint height;

void exchange(inout vec4 i, inout vec4 j) {
    vec4 k;
    k = i;
    i = j;
    j = k;
}

void bitonic_sort() {
    if(gl_GlobalInvocationID.y < height/2 && gl_GlobalInvocationID.x < width) {
        int index = int(gl_GlobalInvocationID.y);
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
        int line_index = inner_block_index + inner_block_offset;

        int x = int(gl_GlobalInvocationID.x);
        int final_index = line_index * int(width) + x;
        int final_index_step = (line_index + step) * int(width) + x;

        if(length(values[final_index]) * up > length(values[final_index_step]) * up) {
            exchange(values[final_index], values[final_index_step]);
        }
    }
}

void main() {
    bitonic_sort();
}
