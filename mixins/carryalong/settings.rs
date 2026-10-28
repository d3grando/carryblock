// native_hooks::Game::native_settings (a hook point): Carry Along's settings page.
fn carryalong_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == crate::game::carryalong::ID {
        buttons.extend(game.carryalong_settings());
    }
}
