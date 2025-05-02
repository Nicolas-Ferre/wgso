#init init()
#run<42> test_compute(mode=mode0)
#run<-42> test_compute(mode=mode1)
#init test_compute(mode=mode0)

var<storage, read_write> mode0: u32;
var<storage, read_write> mode1: u32;
