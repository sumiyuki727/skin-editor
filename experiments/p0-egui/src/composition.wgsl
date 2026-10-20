struct Op { header: vec4<u32>, x: vec4<f32>, y: vec4<f32>, padding: vec4<f32> }
@group(0) @binding(0) var<storage, read> ops: array<Op>;
@group(0) @binding(1) var<storage, read> pixels: array<u32>;
@group(0) @binding(2) var<storage, read_write> output: array<u32>;
@group(0) @binding(3) var<uniform> dimensions: vec4<u32>;
fn decode_channel(v:f32)->f32 {
    if v <= 0.04045 { return v/12.92; }
    return pow((v+0.055)/1.055,2.4);
}
fn encode_channel(v:f32)->f32 {
    if v <= 0.0031308 { return 12.92*v; }
    return 1.055*pow(v,1.0/2.4)-0.055;
}
fn texel(op:Op, q:vec2<i32>)->vec4<f32> {
    let p=clamp(q,vec2<i32>(0),vec2<i32>(op.header.zw)-vec2<i32>(1));
    let packed=pixels[op.header.y+u32(p.y)*op.header.z+u32(p.x)];
    let rgba=vec4<f32>(f32(packed&255u),f32((packed>>8u)&255u),f32((packed>>16u)&255u),f32(packed>>24u))/255.0;
    return vec4<f32>(decode_channel(rgba.r)*rgba.a,decode_channel(rgba.g)*rgba.a,decode_channel(rgba.b)*rgba.a,rgba.a);
}
fn sample_image(op:Op, raw:vec2<f32>)->vec4<f32> {
    let nearest=round(raw);
    let p=select(raw,nearest,abs(raw-nearest)<=vec2<f32>(0.0001));
    if any(p<vec2<f32>(0.0)) || any(p>=vec2<f32>(op.header.zw)) { return vec4<f32>(0.0); }
    if op.header.x==0u { return texel(op,vec2<i32>(floor(p))); }
    let q=p-vec2<f32>(0.5);
    let i=vec2<i32>(floor(q));
    let f=fract(q);
    return mix(mix(texel(op,i),texel(op,i+vec2<i32>(1,0)),f.x),
               mix(texel(op,i+vec2<i32>(0,1)),texel(op,i+vec2<i32>(1,1)),f.x),f.y);
}
fn over(dst:vec4<f32>,src:vec4<f32>)->vec4<f32> { return src+dst*(1.0-src.a); }
fn pack(v:vec4<f32>)->u32 {
    let alpha=u32(round(clamp(v.a,0.0,1.0)*255.0));
    if alpha==0u { return 0u; }
    let rgb=clamp(v.rgb/v.a,vec3<f32>(0.0),vec3<f32>(1.0));
    let r=u32(round(encode_channel(rgb.r)*255.0));
    let g=u32(round(encode_channel(rgb.g)*255.0));
    let b=u32(round(encode_channel(rgb.b)*255.0));
    return r | (g<<8u) | (b<<16u) | (alpha<<24u);
}
@compute @workgroup_size(8,8)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if id.x>=dimensions.x || id.y>=dimensions.y { return; }
    var stack:array<vec4<f32>,9>;
    var depth=0u;
    let point=vec3<f32>(vec2<f32>(id.xy)+vec2<f32>(0.5),1.0);
    for(var index=0u;index<dimensions.z;index++) {
        let op=ops[index];
        if op.header.x==1u {
            depth++; stack[depth]=vec4<f32>(0.0);
        } else if op.header.x==2u {
            let src=stack[depth]*op.x.w;
            depth--; stack[depth]=over(stack[depth],src);
        } else {
            let local=vec2<f32>(dot(op.x.xyz,point),dot(op.y.xyz,point));
            stack[depth]=over(stack[depth],sample_image(op,local)*op.x.w);
        }
    }
    output[id.y*dimensions.x+id.x]=pack(stack[0]);
}
