use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

/// 最小窗口壳的配置。
///
/// M1-S1 只负责创建和关闭窗口，不持有任何 Vulkan 对象。
#[derive(Debug, Clone, PartialEq)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Learn Vulkan Renderer - M1-S1".to_owned(),
            width: 1280,
            height: 720,
        }
    }
}

impl WindowConfig {
    pub fn attributes(&self) -> WindowAttributes {
        Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(LogicalSize::new(self.width, self.height))
    }
}

/// 运行 M1-S1 的桌面窗口 demo。
///
/// 返回值使用 `Result`，让后续 M1-S2/M1-S3 可以自然接入 surface 和 Vulkan
/// 初始化失败路径，而不是在入口处直接 panic。
pub fn run_window_shell(config: WindowConfig) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    let mut app = WindowShell::new(config);

    event_loop.run_app(&mut app)
}

#[derive(Debug)]
struct WindowShell {
    config: WindowConfig,
    window: Option<Window>,
}

impl WindowShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            window: None,
        }
    }
}

impl ApplicationHandler for WindowShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = event_loop
            .create_window(self.config.attributes())
            .expect("failed to create M1-S1 desktop window");

        println!(
            "M1-S1 window created: '{}' ({}x{})",
            self.config.title, self.config.width, self.config.height
        );

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        if matches!(event, WindowEvent::CloseRequested) {
            println!("M1-S1 window close requested");
            event_loop.exit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WindowConfig;

    #[test]
    fn default_window_config_is_desktop_sized() {
        let config = WindowConfig::default();

        assert_eq!(config.title, "Learn Vulkan Renderer - M1-S1");
        assert_eq!((config.width, config.height), (1280, 720));
    }
}
