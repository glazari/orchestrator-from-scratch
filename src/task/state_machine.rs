use super::task::State;

type S = State;

pub fn state_transition_map(state: State) -> &'static [State] {
    match state {
        S::Pending => &[S::Scheduled],
        S::Scheduled => &[S::Scheduled, S::Running, S::Failed],
        S::Running => &[S::Running, S::Completed, S::Failed],
        S::Completed => &[],
        S::Failed => &[],
    }
}

pub fn is_valid_transition(from: State, to: State) -> bool {
    state_transition_map(from).contains(&to)
}
