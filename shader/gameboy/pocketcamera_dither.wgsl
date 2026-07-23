// Compute shader

const TILES_PER_ROW: u32 = 16;
const TILES_PER_COLUMN: u32 = 14;

const TILE_X_OFFSET: u32 = 2;
const PACKED_U32_PER_ROW: u32 = TILE_X_OFFSET * TILES_PER_ROW;
const TILE_Y_OFFSET: u32 = PACKED_U32_PER_ROW * 8;
const INVOCATION_OFFSET: u32 = PACKED_U32_PER_ROW * 2;

const PACKED_U32_PER_TILE: u32 = 4;
const PACKED_U32_PER_TILE_ROW: u32 = PACKED_U32_PER_TILE * TILES_PER_ROW;

const BUFFER_SIZE: u32 = TILES_PER_ROW * TILES_PER_COLUMN * PACKED_U32_PER_TILE;

@group(0) @binding(0) var<storage, read> dither_matrix: array<array<vec4<u32>, 4>, 3>; // indexed by [threshold index][y][x]
@group(0) @binding(1) var<storage, read> sensor_buffer: array<u32, BUFFER_SIZE>; // Packed U8
@group(0) @binding(2) var<storage, write> image_buffer: array<u32, BUFFER_SIZE>; // Packed U8

@compute @workgroup_size(1, 4, 1) // Computes one tile (comprised of 4 8x2 chunks)
fn apply_dither_matrix(@builtin(workgroup_id) workgroup_id : vec3<u32>, @builtin(local_invocation_id) local_invocation_id : vec3<u32>) {
    let sbuffer_base = (workgroup_id.x * TILE_X_OFFSET) + (workgroup_id.y * TILE_Y_OFFSET) + (local_invocation_id.y * INVOCATION_OFFSET);
    let ibuffer_index = (workgroup_id.x * PACKED_U32_PER_TILE) + (workgroup_id.y * PACKED_U32_PER_TILE_ROW) + (local_invocation_id.y);
    let dmatrix_base = (local_invocation_id.y * 2) % 4;

    let packed_result: u32;
    // Iterate over both 8-pixel rows of workgroup
    for (y: u32 = 0; y < 2; y++) {
        let sbuffer_index = sbuffer_base + (y * PACKED_U32_PER_ROW);
        let dmatrix_index = dmatrix_base + y;
        let sensor_value = array<vec4<u32>, 2>(unpack4xU8(sensor_buffer[sbuffer_index]), unpack4xU8(sensor_buffer[sbuffer_index + 1]));
        
        let bitplane: vec2<u32>;
        // Iterate over the pixels in both 4xU8s
        for (x: u32 = 0; x < 2; x++) {
            // Test against thresholds and find final shade
            let b01 = sensor_value[x] < dither_matrix[0][dmatrix_index];
            let b10 = b01 & (sensor_value[x] < dither_matrix[1][dmatrix_index]);
            let b11 = b10 & (sensor_value[x] < dither_matrix[2][dmatrix_index]);
            let shade = u32(b01) + u32(b10) + u32(b11);
            
            // Split into bitplanes
            let partial: vec2<u32>
            partial[0] = (shade[0] & 1) << 3
                   | (shade[1] & 1) << 2
                   | (shade[2] & 1) << 1
                   | (shade[3] & 1);
            partial[1] = (shade[0] & 2) << 3
                   | (shade[1] & 2) << 2
                   | (shade[2] & 2) << 1
                   | (shade[3] & 2);

            bitplane = (bitplane << 4) | partial;
        }

        // Composite with final output
        packed_result = (unpacked_result << 16)
                        | (bitplane[0] << 8) 
                        | bitplane[1];
    }

    image_buffer[ibuffer_index] = packed_result;
}