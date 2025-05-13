//! Storage used retrieve or send information to the CPU.

#import ~.storage_types

/// Main storage variable of the standard library.
var<storage, read_write> std_: Std;
