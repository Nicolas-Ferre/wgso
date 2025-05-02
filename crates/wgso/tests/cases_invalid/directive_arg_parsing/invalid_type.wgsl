#shader<compute> invalid_type
#run invalid_type(value=buffer_invalid_type)

var<storage, read_write> buffer_invalid_type: i32;

var<uniform> value: u32;
