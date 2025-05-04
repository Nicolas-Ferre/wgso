#import ~.main

const TRIANGLE_COUNT = 3;

var<storage, read_write> triangles: TriangleState;

struct TriangleState {
    instances: array<Triangle, TRIANGLE_COUNT>,
}
