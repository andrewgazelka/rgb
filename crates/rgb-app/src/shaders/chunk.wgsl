struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;
@group(1) @binding(1)
var s_atlas: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
}

struct InstanceInput {
    @location(2) world_pos: vec2<f32>,
    @location(3) atlas_uv: vec2<f32>,
    @location(4) region_color: vec3<f32>,
    @location(5) chunk_in_region: vec2<f32>, // (x, y) position within region 0-3
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) region_color: vec3<f32>,
    @location(3) chunk_in_region: vec2<f32>,
}

const CHUNK_SIZE: f32 = 64.0; // 16 cells * 4 pixels per cell
const ATLAS_CHUNK_SIZE: f32 = 1.0 / 256.0; // 256 chunks in atlas

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Scale unit quad to chunk pixel size and position in world
    let world_position = instance.world_pos + vertex.position * CHUNK_SIZE;

    out.clip_position = camera.view_proj * vec4<f32>(world_position, 0.0, 1.0);

    // Map unit UV to atlas region for this chunk
    out.tex_coord = instance.atlas_uv + vertex.tex_coord * ATLAS_CHUNK_SIZE;

    // Pass local position (0-1) for border drawing
    out.local_pos = vertex.tex_coord;
    out.region_color = instance.region_color;
    out.chunk_in_region = instance.chunk_in_region;

    return out;
}

const BORDER_WIDTH: f32 = 0.03; // Border width as fraction of chunk
const REGION_SIZE: f32 = 4.0;   // 4x4 chunks per region

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cell_color = textureSample(t_atlas, s_atlas, in.tex_coord);

    // Check if we're on a region boundary
    let is_left_edge = in.chunk_in_region.x < 0.5;
    let is_right_edge = in.chunk_in_region.x > REGION_SIZE - 1.5;
    let is_bottom_edge = in.chunk_in_region.y < 0.5;
    let is_top_edge = in.chunk_in_region.y > REGION_SIZE - 1.5;

    // Check if pixel is within border area
    let near_left = in.local_pos.x < BORDER_WIDTH;
    let near_right = in.local_pos.x > 1.0 - BORDER_WIDTH;
    let near_bottom = in.local_pos.y < BORDER_WIDTH;
    let near_top = in.local_pos.y > 1.0 - BORDER_WIDTH;

    // Draw border on region edges
    let draw_left_border = is_left_edge && near_left;
    let draw_right_border = is_right_edge && near_right;
    let draw_bottom_border = is_bottom_edge && near_bottom;
    let draw_top_border = is_top_edge && near_top;

    if draw_left_border || draw_right_border || draw_bottom_border || draw_top_border {
        return vec4<f32>(in.region_color, 1.0);
    }

    return cell_color;
}
