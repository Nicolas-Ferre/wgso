#shader<compute> test

@group(0)
@binding(0)
var<storage, read_write> a: f32;
@group(0)
@binding(1)
var<storage, read_write> b: f32;
@group(0)
@binding(2)
var<storage, read_write> c: f32;
@group(0)
@binding(3)
var<storage, read_write> d: f32;
@group(0)
@binding(4)
var<storage, read_write> e: f32;
@group(0)
@binding(5)
var<storage, read_write> f: f32;
@group(0)
@binding(6)
var<storage, read_write> g: f32;
@group(0)
@binding(7)
var<storage, read_write> h: f32;
@group(0)
@binding(8)
var<storage, read_write> i: f32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
}
