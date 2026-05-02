use std::{
    error::Error,
    ffi::CString,
    fmt::{Debug, Display, Formatter},
};

use ash::{Entry, Instance, khr::surface::Instance as SurfaceLoader, vk};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};

use crate::WindowConfig;

#[derive(Debug)]
pub enum SurfaceError {
    EventLoop(winit::error::EventLoopError),
    Window(winit::error::OsError),
    WindowHandle(HandleError),
    VulkanLoader(ash::LoadingError),
    Vulkan(vk::Result),
}

impl Display for SurfaceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoop(error) => write!(formatter, "winit event loop error: {error}"),
            Self::Window(error) => write!(formatter, "winit window error: {error}"),
            Self::WindowHandle(error) => {
                write!(formatter, "raw window/display handle error: {error}")
            }
            Self::VulkanLoader(error) => write!(formatter, "failed to load Vulkan loader: {error}"),
            Self::Vulkan(error) => write!(formatter, "Vulkan error: {error:?}"),
        }
    }
}

impl Error for SurfaceError {}

impl From<winit::error::EventLoopError> for SurfaceError {
    fn from(error: winit::error::EventLoopError) -> Self {
        Self::EventLoop(error)
    }
}

impl From<winit::error::OsError> for SurfaceError {
    fn from(error: winit::error::OsError) -> Self {
        Self::Window(error)
    }
}

impl From<HandleError> for SurfaceError {
    fn from(error: HandleError) -> Self {
        Self::WindowHandle(error)
    }
}

impl From<ash::LoadingError> for SurfaceError {
    fn from(error: ash::LoadingError) -> Self {
        Self::VulkanLoader(error)
    }
}

impl From<vk::Result> for SurfaceError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

/// M1-S2 的最小 surface bootstrap。
///
/// 这个类型只验证 window handle 到 `VkSurfaceKHR` 的路径。正式的
/// `VulkanInstance` 抽象、validation layer 和 debug messenger 留给 M1-S3/M1-S4。
pub struct SurfaceBootstrap {
    _entry: Entry,
    instance: Instance,
    surface_loader: SurfaceLoader,
    surface: vk::SurfaceKHR,
}

impl Debug for SurfaceBootstrap {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SurfaceBootstrap")
            .field("surface", &self.surface)
            .finish_non_exhaustive()
    }
}

impl SurfaceBootstrap {
    pub fn new(window: &Window) -> Result<Self, SurfaceError> {
        let display_handle = window.display_handle()?.as_raw();
        let window_handle = window.window_handle()?.as_raw();

        // SAFETY: `Entry::load` 只从平台 Vulkan loader 加载函数指针。
        // 返回的 `Entry` 不拥有 Vulkan 对象，并且会在 instance 生命周期内保持存活。
        let entry = unsafe { Entry::load()? };

        let extension_names = ash_window::enumerate_required_extensions(display_handle)?;
        let app_name =
            CString::new("learn-vulkan-renderer-m1-s2").expect("static app name has no nul");
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

        // SAFETY: 调用期间 `instance_info` 引用的栈上数据都保持有效，
        // 并且已经启用 `ash-window` 返回的平台 surface 必需扩展。
        let instance = unsafe { entry.create_instance(&instance_info, None)? };
        let surface_loader = SurfaceLoader::new(&entry, &instance);

        // SAFETY: 两个 raw handle 都来自仍然存活的 `winit::Window`。
        // 只要 `SurfaceBootstrap` 存在，这个 window 就仍由应用壳持有。
        // instance 创建时启用了同一个 display handle 对应的 surface 扩展。
        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)?
        };

        Ok(Self {
            _entry: entry,
            instance,
            surface_loader,
            surface,
        })
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }
}

impl Drop for SurfaceBootstrap {
    fn drop(&mut self) {
        // SAFETY: `surface` 由这个 `instance` 创建，且尚未被销毁。
        // 先销毁 surface，再销毁 instance，满足 Vulkan 父子对象生命周期规则。
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

pub fn run_surface_shell(config: WindowConfig) -> Result<(), SurfaceError> {
    let event_loop = EventLoop::new()?;
    let mut app = SurfaceShell::new(config);

    event_loop.run_app(&mut app)?;
    app.result
}

#[derive(Debug)]
struct SurfaceShell {
    config: WindowConfig,
    // 如果事件循环没有调用 `exiting`，字段声明顺序仍会保证 surface 先于 window drop。
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    result: Result<(), SurfaceError>,
}

impl SurfaceShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            surface: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_window_and_surface(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), SurfaceError> {
        if self.surface.is_some() {
            return Ok(());
        }

        if self.window.is_none() {
            self.window = Some(event_loop.create_window(self.config.attributes())?);
        }

        let window = self
            .window
            .as_ref()
            .expect("window was created before surface bootstrap");
        let surface = SurfaceBootstrap::new(window)?;

        println!("M1-S2 surface created: {:?}", surface.surface());
        self.surface = Some(surface);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: SurfaceError) {
        eprintln!("M1-S2 surface bootstrap failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for SurfaceShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_window_and_surface(event_loop) {
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
            println!("M1-S2 window close requested");
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.surface = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.surface = None;
    }
}
