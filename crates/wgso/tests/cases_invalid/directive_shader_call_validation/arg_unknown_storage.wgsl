#shader<compute> arg_unknown_storage
#shader<render, u32> arg_unknown_storage

#init arg_unknown_storage(param=arg_unknown_storage)
#run arg_unknown_storage(param=arg_unknown_storage)
#draw arg_unknown_storage<vertices>(param=arg_unknown_storage)

var<uniform> param: u32;
