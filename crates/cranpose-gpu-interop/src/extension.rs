//! The GDExtension entry point, for every target Godot loads extensions on.

/// Declares `$extension` as the GDExtension entry point of the library that
/// calls it, so the library's own code holds no unsafe block.
///
/// # Safety
///
/// godot-rust needs exactly one `ExtensionLibrary` per dynamic library: call
/// this macro once, in the library's root module.
#[macro_export]
macro_rules! godot_extension {
    ($extension:ident) => {
        struct $extension;

        // SAFETY: the one entry point of this library, as the macro's
        // contract asks of its caller.
        #[::godot::init::gdextension]
        unsafe impl ::godot::init::ExtensionLibrary for $extension {}
    };
}
