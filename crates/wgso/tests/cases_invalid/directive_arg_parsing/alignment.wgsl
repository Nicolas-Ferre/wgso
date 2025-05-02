#shader<compute> alignment
#run alignment(param=buffer_alignment.field2)

var<storage> buffer_alignment: TestStruct;

var<uniform> param: u32;

struct TestStruct {
    field1: u32,
    field2: u32,
}
