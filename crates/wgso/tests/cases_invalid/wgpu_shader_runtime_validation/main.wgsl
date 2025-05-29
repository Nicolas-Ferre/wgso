#mod<compute> test

var<storage, read_write> storage1: f32;
var<storage, read_write> storage2: f32;
var<storage, read_write> storage3: f32;
var<storage, read_write> storage4: f32;
var<storage, read_write> storage5: f32;
var<storage, read_write> storage6: f32;
var<storage, read_write> storage7: f32;
var<storage, read_write> storage8: f32;
var<storage, read_write> storage9: f32;

@compute
@workgroup_size(1, 1, 1)
fn main() {
}
