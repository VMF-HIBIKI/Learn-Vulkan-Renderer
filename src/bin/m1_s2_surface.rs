use learn_vulkan_renderer::{WindowConfig, run_surface_shell};

fn main() -> Result<(), learn_vulkan_renderer::SurfaceError> {
    run_surface_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S2".to_owned(),
        ..WindowConfig::default()
    })
}
