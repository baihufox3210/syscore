#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Status,
}
