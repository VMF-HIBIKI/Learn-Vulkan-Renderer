use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use ash::vk;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    DeviceError, SurfaceBootstrap, VulkanInstanceError, WindowConfig, select_physical_device,
};

#[derive(Debug)]
pub enum SwapchainError {
    Device(DeviceError),
    Vulkan(vk::Result),
    MissingSurfaceFormats,
    MissingPresentModes,
}

impl Display for SwapchainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Device(error) => write!(formatter, "device error: {error}"),
            Self::Vulkan(error) => write!(formatter, "Vulkan error: {error:?}"),
            Self::MissingSurfaceFormats => write!(formatter, "surface has no supported formats"),
            Self::MissingPresentModes => {
                write!(formatter, "surface has no supported present modes")
            }
        }
    }
}

impl Error for SwapchainError {}

impl From<DeviceError> for SwapchainError {
    fn from(error: DeviceError) -> Self {
        Self::Device(error)
    }
}

impl From<VulkanInstanceError> for SwapchainError {
    fn from(error: VulkanInstanceError) -> Self {
        Self::Device(DeviceError::from(error))
    }
}

impl From<vk::Result> for SwapchainError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

#[derive(Debug, Clone)]
pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Debug, Clone, Copy)]
pub struct SwapchainConfig {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
    pub image_count: u32,
}

pub fn query_swapchain_support_details(
    surface_loader: &ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<SwapchainSupportDetails, SwapchainError> {
    // SAFETY: surface belongs to the live instance that enumerated `physical_device`.
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };
    // SAFETY: this only queries formats supported by the device/surface pair.
    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface)? };
    // SAFETY: this only queries present modes supported by the device/surface pair.
    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
    };

    Ok(SwapchainSupportDetails {
        capabilities,
        formats,
        present_modes,
    })
}

pub fn choose_swapchain_config(
    details: &SwapchainSupportDetails,
    window_size: PhysicalSize<u32>,
) -> Result<SwapchainConfig, SwapchainError> {
    let surface_format = choose_surface_format(&details.formats)?;
    let present_mode = choose_present_mode(&details.present_modes)?;
    let extent = choose_extent(&details.capabilities, window_size);
    let image_count = choose_image_count(&details.capabilities);

    Ok(SwapchainConfig {
        surface_format,
        present_mode,
        extent,
        image_count,
    })
}

fn choose_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> Result<vk::SurfaceFormatKHR, SwapchainError> {
    if formats.is_empty() {
        return Err(SwapchainError::MissingSurfaceFormats);
    }

    Ok(formats
        .iter()
        .copied()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(formats[0]))
}

fn choose_present_mode(
    present_modes: &[vk::PresentModeKHR],
) -> Result<vk::PresentModeKHR, SwapchainError> {
    if present_modes.is_empty() {
        return Err(SwapchainError::MissingPresentModes);
    }

    Ok(present_modes
        .iter()
        .copied()
        .find(|present_mode| *present_mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO))
}

fn choose_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    window_size: PhysicalSize<u32>,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }

    vk::Extent2D {
        width: window_size.width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: window_size.height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}

fn choose_image_count(capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
    let preferred_count = capabilities.min_image_count + 1;

    if capabilities.max_image_count > 0 {
        preferred_count.min(capabilities.max_image_count)
    } else {
        preferred_count
    }
}

pub fn run_swapchain_config_shell(config: WindowConfig) -> Result<(), SwapchainError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = SwapchainConfigShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)
        .map_err(DeviceError::from)?;
    app.result
}

#[derive(Debug)]
struct SwapchainConfigShell {
    config: WindowConfig,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    result: Result<(), SwapchainError>,
}

impl SwapchainConfigShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            surface: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_surface_and_choose_config(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), SwapchainError> {
        if self.surface.is_some() {
            return Ok(());
        }

        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(self.config.attributes())
                    .map_err(VulkanInstanceError::from)
                    .map_err(DeviceError::from)?,
            );
        }

        let window = self
            .window
            .as_ref()
            .expect("window was created before swapchain config query");
        let surface = SurfaceBootstrap::new(window).map_err(DeviceError::from)?;
        let selected_device = select_physical_device(
            surface.vulkan_instance(),
            surface.surface_loader(),
            surface.surface(),
        )?;
        let details = query_swapchain_support_details(
            surface.surface_loader(),
            selected_device.info.handle,
            surface.surface(),
        )?;
        let config = choose_swapchain_config(&details, window.inner_size())?;

        println!("M1-S9 selected device: {}", selected_device.info.name);
        println!(
            "M1-S9 swapchain config: format={:?}, color_space={:?}, present_mode={:?}, extent={}x{}, images={}",
            config.surface_format.format,
            config.surface_format.color_space,
            config.present_mode,
            config.extent.width,
            config.extent.height,
            config.image_count
        );

        self.surface = Some(surface);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: SwapchainError) {
        eprintln!("M1-S9 swapchain config failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for SwapchainConfigShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_surface_and_choose_config(event_loop) {
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
            println!("M1-S9 window close requested");
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
