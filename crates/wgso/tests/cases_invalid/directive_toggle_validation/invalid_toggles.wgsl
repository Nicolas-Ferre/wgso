#mod main
#toggle<state.valid_condition> not_matching_mod_prefix.missing
#toggle<state.valid_condition> ~.missing
#toggle<state.invalid_type> ~.toggle
#toggle<internal> ~.other_toggle

#mod toggle

#mod state
#import ~.toggle

var<storage, read_write> state: State;

struct State {
    valid_condition: u32,
    invalid_type: i32,
}

#mod other_toggle

var<storage, read_write> internal: u32;
