var<storage, read_write> buffer: u32;

fn increment(value: ptr<storage, u32>) {
    *value += 1;
}
