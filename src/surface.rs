use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use ash::{khr::surface::Instance as SurfaceLoader, vk};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};

use crate::{VulkanInstance, VulkanInstanceError, WindowConfig};

#[derive(Debug)]
pub enum SurfaceError {
    EventLoop(winit::error::EventLoopError),
    Window(winit::error::OsError),
    WindowHandle(HandleError),
    Instance(VulkanInstanceError),
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
            Self::Instance(error) => write!(formatter, "Vulkan instance error: {error}"),
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

impl From<VulkanInstanceError> for SurfaceError {
    fn from(error: VulkanInstanceError) -> Self {
        Self::Instance(error)
    }
}

impl From<vk::Result> for SurfaceError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

/// M1-S2/M1-S3 的最小 surface bootstrap。
///
/// surface 仍归这个类型管理；`Entry` 与 `VkInstance` 的 owner 已经在 M1-S3
/// 提升为 `VulkanInstance`，后续 validation/debug utils 会继续接入那里。
pub struct SurfaceBootstrap {
    _vulkan_instance: VulkanInstance,
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

        let vulkan_instance =
            VulkanInstance::new_for_display(display_handle, "learn-vulkan-renderer-m1-s2")?;
        let surface_loader = SurfaceLoader::new(vulkan_instance.entry(), vulkan_instance.handle());

        // SAFETY: both raw handles come from the live `winit::Window`, and that window remains
        // owned by the application shell while this `SurfaceBootstrap` exists. The instance was
        // created with the extensions returned for the same display handle.
        let surface = unsafe {
            ash_window::create_surface(
                vulkan_instance.entry(),
                vulkan_instance.handle(),
                display_handle,
                window_handle,
                None,
            )?
        };

        Ok(Self {
            _vulkan_instance: vulkan_instance,
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
        // SAFETY: `surface` was created from this `instance` and has not been destroyed yet.
        // Destroying the surface before the instance preserves Vulkan parent/child lifetime rules.
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
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
    // Drop surface before window if the event loop exits without calling `exiting`.
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
