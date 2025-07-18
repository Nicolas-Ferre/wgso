#mod main
#import objects.rectangle.compute
#import _.std.io.main
#import _.std.vertex.model

#init ~.init()
#run ~.update()
#draw objects.rectangle.render<main.rectangle_vertices, main.keyboard_rectangle>(surface=std_.surface)
#draw objects.rectangle.render<main.rectangle_vertices, main.mouse_rectangle>(surface=std_.surface)
#draw objects.rectangle.render<main.rectangle_vertices, main.touch_rectangles>(surface=std_.surface)

struct Main {
    rectangle_vertices: array<Vertex, 6>,
    keyboard_rectangle: Rectangle,
    mouse_rectangle: Rectangle,
    touch_rectangles: array<Rectangle, MAX_FINGER_COUNT>,
}

var<storage, read_write> main: Main;

#shader<compute> init
#import ~.main

@compute
@workgroup_size(1, 1, 1)
fn main_() {
    main.rectangle_vertices = rectangle_vertices();
    main.keyboard_rectangle = init_rectangle(vec2f(0, 0));
    main.mouse_rectangle = init_rectangle(HIDDEN_RECT_POSITION);
    for (var i = 0u; i < MAX_FINGER_COUNT; i++) {
        main.touch_rectangles[i] = init_rectangle(HIDDEN_RECT_POSITION);
    }
}

#shader<compute> update
#import ~.main

@compute
@workgroup_size(1, 1, 1)
fn main_() {
    main.keyboard_rectangle = update_rectangle_with_keyboard(main.keyboard_rectangle);
    main.mouse_rectangle = update_rectangle_with_mouse(main.mouse_rectangle);
    for (var i = 0u; i < MAX_FINGER_COUNT; i++) {
        main.touch_rectangles[i] = update_rectangle_with_touch(main.touch_rectangles[i], i);
    }
}
