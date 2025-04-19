#shader<compute> test
#run test(param=buffer.field2)

var<storage> buffer: TestStruct;

var<uniform> param: u32;

struct TestStruct {
    field1: u32,
    field2: u32,
}
