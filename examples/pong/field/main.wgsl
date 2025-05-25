#init init_field()
#run update_field()
#draw<1000> field<field.vertices, field.instance>(surface=surface)

#import _.std.vertex

const FIELD_SIZE = vec2f(1.9, 1.2);

struct FieldData {
    vertices: array<Vertex, 6>,
    instance: Field,
}

struct Field {
    time: f32,
}
