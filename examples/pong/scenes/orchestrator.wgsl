#mod main

#toggle<orchestrator.is_menu_enabled> scenes.menu
#toggle<orchestrator.is_game_enabled> scenes.game
#init ~.init()

const MENU_SCENE_ID = 0;
const GAME_SCENE_ID = 1;

struct Orchestrator {
    is_menu_enabled: u32,
    is_game_enabled: u32,
    is_multiplayer: u32,
}

var<storage, read_write> orchestrator: Orchestrator;

fn start_singleplayer() {
    orchestrator.is_menu_enabled = u32(false);
    orchestrator.is_game_enabled = u32(true);
    orchestrator.is_multiplayer = u32(false);
}

fn start_multiplayer() {
    orchestrator.is_menu_enabled = u32(false);
    orchestrator.is_game_enabled = u32(true);
    orchestrator.is_multiplayer = u32(true);
}

#shader<compute> init
#import ~.main

@compute
@workgroup_size(1, 1, 1)
fn main() {
    orchestrator.is_menu_enabled = u32(true);
}
