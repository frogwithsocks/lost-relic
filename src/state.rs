#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum GameState {
    MainMenu,
    Play,
    Pause,
    Death,
}