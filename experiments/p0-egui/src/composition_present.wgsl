// Presentation adapter only. Export's straight RGBA8 result is never modified.
@group(0) @binding(0) var<storage,read> result:array<u32>;
@group(0) @binding(1) var display_image:texture_storage_2d<rgba8unorm,write>;
fn decode(v:f32)->f32 {
    if v<=0.04045 {return v/12.92;} return pow((v+0.055)/1.055,2.4);
}
fn encode(v:f32)->f32 {
    if v<=0.0031308 {return 12.92*v;} return 1.055*pow(v,1.0/2.4)-0.055;
}
@compute @workgroup_size(8,8)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    let size=textureDimensions(display_image);
    if any(id.xy>=size) {return;}
    let p=result[id.y*size.x+id.x];
    let a=f32(p>>24u)/255.0;
    let rgb=vec3<f32>(f32(p&255u),f32((p>>8u)&255u),f32((p>>16u)&255u))/255.0;
    // Fixed editor canvas [18,22,30], composed in linear light to opaque sRGB.
    let color=vec3<f32>(decode(rgb.r),decode(rgb.g),decode(rgb.b))*a
        +vec3<f32>(decode(18.0/255.0),decode(22.0/255.0),decode(30.0/255.0))*(1.0-a);
    textureStore(display_image,vec2<i32>(id.xy),vec4<f32>(encode(color.r),encode(color.g),encode(color.b),1.0));
}
