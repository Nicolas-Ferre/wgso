#shader<compute> duplicated
#run duplicated(param1=buffer_duplicated, param2=buffer_duplicated, param1=buffer_duplicated)

var<storage, read_write> buffer_duplicated: u32;

var<uniform> param1: u32;
var<uniform> param2: u32;
