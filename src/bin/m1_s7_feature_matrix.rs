use learn_vulkan_renderer::{WindowConfig, run_feature_matrix_shell};

fn main() -> Result<(), learn_vulkan_renderer::DeviceError> {
    run_feature_matrix_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S7".to_owned(),
        ..WindowConfig::default()
    })
}
