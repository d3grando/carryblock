// native_hooks::Game::native_command (a hook point): run `/carryalong`, leave
// everything else to the game (and to other mods).
fn carryalong_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if !*ran && game.carryalong_command(command) {
        *ran = true;
    }
}
