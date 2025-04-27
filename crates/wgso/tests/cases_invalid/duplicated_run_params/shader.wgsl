#shader<compute> test
#run test(param1=a, param2=b, param1=c)

var<storage, read_write> a: u32;
var<storage, read_write> b: u32;
var<storage, read_write> c: u32;

var<uniform> param1: u32;
var<uniform> param2: u32;
