#mod main
#import constant.main
#import objects.player_icon.compute
#import _.std.ui.compute

#init ~.init()
#run ~.update()
#draw objects.button.render<vertices.rectangle, menu.singleplayer_button>(surface=std_.surface)
#draw objects.player_icon.render<vertices.rectangle, menu.singleplayer_icon>(surface=std_.surface)
#draw objects.button.render<vertices.rectangle, menu.multiplayer_button>(surface=std_.surface)
#draw objects.player_icon.render<vertices.rectangle, menu.multiplayer_icon2>(surface=std_.surface)
#draw objects.player_icon.render<vertices.rectangle, menu.multiplayer_icon1>(surface=std_.surface)

const BUTTON_Z = 0.9;
const ICON_Z_MAX = 0.8;
const ICON_Z_MIN = 0.7;

struct Menu {
    singleplayer_button: UiButton,
    singleplayer_icon: PlayerIcon,
    multiplayer_button: UiButton,
    multiplayer_icon1: PlayerIcon,
    multiplayer_icon2: PlayerIcon,
}

var<storage, read_write> menu: Menu;

#shader<compute> init
#import ~.main

const ICON_SIZE = 0.8;
const BUTTON_SIZE = vec2f(0.85, 0.8);

@compute
@workgroup_size(1, 1, 1)
fn main() {
    menu = Menu(
        init_ui_button(vec3f(-0.5, 0, BUTTON_Z), BUTTON_SIZE),
        init_player_icon(vec3f(-0.5, 0, ICON_Z_MIN), ICON_SIZE),
        init_ui_button(vec3f(0.5, 0, BUTTON_Z), BUTTON_SIZE),
        init_player_icon(vec3f(0.45, 0, ICON_Z_MIN), ICON_SIZE),
        init_player_icon(vec3f(0.55, 0, ICON_Z_MAX), ICON_SIZE),
    );
}

#shader<compute> update
#import ~.main
#import constant.main
#import scenes.orchestrator.main
#import _.std.math.matrix
#import _.std.vertex.transform

@compute
@workgroup_size(1, 1, 1)
fn main() {
    let scale_factor = vec3f(scale_factor(std_.surface.size, VISIBLE_AREA_MIN_SIZE), 1);
    let view_mat_arr = mat4x4f_to_array(view_mat(vec3f(0, 0, 0), scale_factor, DEFAULT_QUAT));
    menu.singleplayer_button = update_ui_button(menu.singleplayer_button, view_mat_arr);
    menu.singleplayer_icon.button_state = menu.singleplayer_button.state;
    menu.multiplayer_button = update_ui_button(menu.multiplayer_button, view_mat_arr);
    menu.multiplayer_icon1.button_state = menu.multiplayer_button.state;
    menu.multiplayer_icon2.button_state = menu.multiplayer_button.state;
    if menu.singleplayer_button.state == BUTTON_STATE_RELEASED {
        start_singleplayer();
    } else if menu.multiplayer_button.state == BUTTON_STATE_RELEASED {
        start_multiplayer();
    }
}
