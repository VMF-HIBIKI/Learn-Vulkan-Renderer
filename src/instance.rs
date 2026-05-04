use std::{
    error::Error,
    ffi::CString,
    fmt::{Debug, Display, Formatter},
};

use ash::{Entry, Instance, vk};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HandleError, HasDisplayHandle, RawDisplayHandle},
    window::{Window, WindowId},
};

use crate::WindowConfig;

#[derive(Debug)]
pub enum VulkanInstanceError {
    EventLoop(winit::error::EventLoopError),
    Window(winit::error::OsError),
    WindowHandle(HandleError),
    VulkanLoader(ash::LoadingError),
    Vulkan(vk::Result),
}

impl Display for VulkanInstanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoop(error) => write!(formatter, "winit event loop error: {error}"),
            Self::Window(error) => write!(formatter, "winit window error: {error}"),
            Self::WindowHandle(error) => {
                write!(formatter, "raw display handle error: {error}")
            }
            Self::VulkanLoader(error) => write!(formatter, "failed to load Vulkan loader: {error}"),
            Self::Vulkan(error) => write!(formatter, "Vulkan error: {error:?}"),
        }
    }
}

impl Error for VulkanInstanceError {}

impl From<winit::error::EventLoopError> for VulkanInstanceError {
    fn from(error: winit::error::EventLoopError) -> Self {
        Self::EventLoop(error)
    }
}

impl From<winit::error::OsError> for VulkanInstanceError {
    fn from(error: winit::error::OsError) -> Self {
        Self::Window(error)
    }
}

impl From<HandleError> for VulkanInstanceError {
    fn from(error: HandleError) -> Self {
        Self::WindowHandle(error)
    }
}

impl From<ash::LoadingError> for VulkanInstanceError {
    fn from(error: ash::LoadingError) -> Self {
        Self::VulkanLoader(error)
    }
}

impl From<vk::Result> for VulkanInstanceError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

/// M1-S3 的 Vulkan instance owner。
///
/// 这个类型只负责 `Entry` 与 `VkInstance` 生命周期。后续 M1-S4 会在这里
/// 接入 validation layer 和 debug messenger，surface/device/swapchain 继续保持在外层。
pub struct VulkanInstance {
    entry: Entry,
    instance: Instance,
    enabled_extension_count: usize,
}

impl Debug for VulkanInstance {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VulkanInstance")
            .field("enabled_extension_count", &self.enabled_extension_count)
            .finish_non_exhaustive()
    }
}

impl VulkanInstance {
    pub fn new_for_display(
        display_handle: RawDisplayHandle,
        app_name: &str,
    ) -> Result<Self, VulkanInstanceError> {
        // SAFETY: `Entry::load` 只从平台 Vulkan loader 加载函数指针。
        // 返回的 `Entry` 会由 `VulkanInstance` 持有，生命周期覆盖 `Instance`。
        let entry = unsafe { Entry::load()? };

        let extension_names = ash_window::enumerate_required_extensions(display_handle)?;
        let app_name = CString::new(app_name).expect("static app name has no nul");
        let engine_name =
            CString::new("learn-vulkan-renderer").expect("static engine name has no nul");
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));
        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extension_names);

        // SAFETY: 调用期间 `instance_info` 引用的栈上数据都保持有效。
        // 已启用 `ash-window` 根据 display handle 返回的平台 surface 必需扩展。
        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        Ok(Self {
            entry,
            instance,
            enabled_extension_count: extension_names.len(),
        })
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    pub fn handle(&self) -> &Instance {
        &self.instance
    }

    pub fn enabled_extension_count(&self) -> usize {
        self.enabled_extension_count
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        // SAFETY: `VulkanInstance` 是 `VkInstance` 的唯一 owner。
        // 调用者必须在 drop 前释放 surface/device 等 instance child objects。
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

pub fn run_instance_shell(config: WindowConfig) -> Result<(), VulkanInstanceError> {
    let event_loop = EventLoop::new()?;
    let mut app = InstanceShell::new(config);

    event_loop.run_app(&mut app)?;
    app.result
}

#[derive(Debug)]
struct InstanceShell {
    config: WindowConfig,
    vulkan_instance: Option<VulkanInstance>,
    window: Option<Window>,
    result: Result<(), VulkanInstanceError>,
}

impl InstanceShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            vulkan_instance: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_window_and_instance(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), VulkanInstanceError> {
        if self.vulkan_instance.is_some() {
            return Ok(());
        }

        if self.window.is_none() {
            self.window = Some(event_loop.create_window(self.config.attributes())?);
        }

        let window = self
            .window
            .as_ref()
            .expect("window was created before Vulkan instance bootstrap");
        let display_handle = window.display_handle()?.as_raw();
        let vulkan_instance =
            VulkanInstance::new_for_display(display_handle, "learn-vulkan-renderer-m1-s3")?;

        println!(
            "M1-S3 Vulkan instance created with {} required surface extensions",
            vulkan_instance.enabled_extension_count()
        );
        self.vulkan_instance = Some(vulkan_instance);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: VulkanInstanceError) {
        eprintln!("M1-S3 Vulkan instance bootstrap failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for InstanceShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_window_and_instance(event_loop) {
            self.record_error_and_exit(event_loop, error);
        }
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
            println!("M1-S3 window close requested");
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.vulkan_instance = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.vulkan_instance = None;
    }
}
