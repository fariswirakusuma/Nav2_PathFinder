use bevy::prelude::*;
use crate::config::states::AppState;

#[derive(Resource, Default)]
pub struct NavStack {
    pub history: Vec<AppState>,
}

pub fn push_state(
    current_state: AppState, 
    new_state: AppState,    
    next_state: &mut NextState<AppState>, 
    nav_stack: &mut NavStack,
) {
    nav_stack.history.push(current_state);
    next_state.set(new_state);
}

pub fn pop_state(
    next_state: &mut NextState<AppState>, 
    nav_stack: &mut NavStack,
) {
    if let Some(prev_state) = nav_stack.history.pop() {
        next_state.set(prev_state);
    }
}