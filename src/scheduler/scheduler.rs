pub trait Secheduler {
    fn select_candidate_nodes(&self) -> ();
    fn score(&self) -> ();
    fn pick(&self) -> ();
}
