#shader<compute> test
#run test(param=buffer.unknown)

var<storage> buffer: TestStruct;

var<uniform> param: u32;

struct TestStruct {
    field: u32,
}
