// native_hooks::Game::native_mods (a hook point): Carry Along's row in Options > Mods.
fn carryalong_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::carryalong as ca;
    mods.push(crate::game::native_hooks::NativeMod {
        id: ca::ID.into(),
        title: format!("Carry Along {}", ca::VERSION),
        description: ca::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
