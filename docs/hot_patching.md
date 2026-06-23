How to run
dx serve --hot-patch
This starts your app with hotpatching enabled. Edit any system's body, save the file, and see changes reflected in ~500ms without restarting the app. Edit any asset (textures, WGSL shader) in assets/ and those reload automatically too.
How to mark systems for hotpatching
Add use bevy::app::hotpatch::hot; then annotate systems:
#[hot]
fn my_system(query: Query<&Transform>, ...) {
    // edit this at runtime
}
Known limitations
- Don't use #[hot] on systems with EventReader, Local<T>, or Added<T>/Changed<T> filters
- Changing type/struct definitions still requires a full restart
- #[hot] systems run as exclusive (not in parallel with other hotpatched systems)
- The #[hot] attribute compiles away harmlessly in release builds