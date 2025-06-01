#mod function
#import inner.inner.storage

fn increment() {
    counter += 1;
}

// To test circular imports:
#mod storage
#import inner.inner.function

var<storage, read_write> counter: u32;
