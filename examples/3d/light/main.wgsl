#init ~.init.init_light()
#draw ~.draw.light<cubes.vertices, light.point>(camera=camera)

const POINT_LIGHT_SIZE = vec3f(0.01, 0.01, 0.01);

struct Light {
    ambiant: AmbiantLight,
    point: PointLight,
}

struct PointLight {
    position: vec3f,
    color: vec3f,
}

struct AmbiantLight {
    strength: f32,
}
