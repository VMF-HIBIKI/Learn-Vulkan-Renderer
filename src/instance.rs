use std::{
    error::Error,
    ffi::{CStr, CString, c_void},
    fmt::{Debug, Display, Formatter},
};

use ash::{Entry, Instance, ext::debug_utils::Instance as DebugUtilsLoader, vk};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HandleError, HasDisplayHandle, RawDisplayHandle},
    window::{Window, WindowId},
};

use crate::WindowConfig;

const VALIDATION_LAYER_NAME: &str = "VK_LAYER_KHRONOS_validation";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VulkanInstanceConfig {
    pub enable_validation: bool,
}

impl Default for VulkanInstanceConfig {
    fn default() -> Self {
        Self {
            enable_validation: cfg!(debug_assertions),
        }
    }
}

/// M1-S3 的 Vulkan instance owner。
///
/// 这个类型负责 `Entry`、`VkInstance` 和 instance-level debug messenger 生命周期。
/// surface/device/swapchain 继续保持在外层，避免 Vulkan 根对象反向依赖下游资源。
pub struct VulkanInstance {
    entry: Entry,
    instance: Instance,
    debug_utils_loader: Option<DebugUtilsLoader>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    enabled_extension_count: usize,
    validation_enabled: bool,
}

impl Debug for VulkanInstance {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VulkanInstance")
            .field("enabled_extension_count", &self.enabled_extension_count)
            .field("validation_enabled", &self.validation_enabled)
            .finish_non_exhaustive()
    }
}

impl VulkanInstance {
    pub fn new_for_display(
        display_handle: RawDisplayHandle,
        app_name: &str,
    ) -> Result<Self, VulkanInstanceError> {
        Self::new_for_display_with_config(display_handle, app_name, VulkanInstanceConfig::default())
    }

    pub fn new_for_display_with_config(
        display_handle: RawDisplayHandle,
        app_name: &str,
        config: VulkanInstanceConfig,
    ) -> Result<Self, VulkanInstanceError> {
        // SAFETY: `Entry::load` 只从平台 Vulkan loader 加载函数指针。
        // 返回的 `Entry` 会由 `VulkanInstance` 持有，生命周期覆盖 `Instance`。
        let entry = unsafe { Entry::load()? };

        let validation_layer_name =
            CString::new(VALIDATION_LAYER_NAME).expect("static layer name has no nul");
        let validation_enabled =
            config.enable_validation && instance_layer_available(&entry, &validation_layer_name)?;

        if config.enable_validation && !validation_enabled {
            eprintln!(
                "Vulkan validation layer '{VALIDATION_LAYER_NAME}' is unavailable; continuing without validation"
            );
        }

        let mut extension_names =
            ash_window::enumerate_required_extensions(display_handle)?.to_vec();
        let debug_utils_available =
            instance_extension_available(&entry, ash::ext::debug_utils::NAME)?;

        if validation_enabled && debug_utils_available {
            extension_names.push(ash::ext::debug_utils::NAME.as_ptr());
        } else if validation_enabled {
            eprintln!("Vulkan debug utils extension is unavailable; validation logging disabled");
        }

        let validation_enabled = validation_enabled && debug_utils_available;
        let layer_names = if validation_enabled {
            vec![validation_layer_name.as_ptr()]
        } else {
            Vec::new()
        };
        let app_name = CString::new(app_name).expect("static app name has no nul");
        let engine_name =
            CString::new("learn-vulkan-renderer").expect("static engine name has no nul");
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));
        let mut debug_messenger_info = debug_messenger_create_info();
        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names);

        if validation_enabled {
            instance_info = instance_info.push_next(&mut debug_messenger_info);
        }

        // SAFETY: 调用期间 `instance_info` 引用的栈上数据都保持有效。
        // 已启用 `ash-window` 根据 display handle 返回的平台 surface 必需扩展；
        // validation/debug utils 的名字来自本函数内仍存活的 `CString` 和静态扩展名。
        let instance = unsafe { entry.create_instance(&instance_info, None)? };
        let (debug_utils_loader, debug_messenger) = if validation_enabled {
            let debug_utils_loader = DebugUtilsLoader::new(&entry, &instance);

            // SAFETY: `debug_messenger_info` 的 callback 是静态函数，loader 与 instance
            // 会由 `VulkanInstance` 持有并晚于 messenger 销毁。
            let debug_messenger = unsafe {
                debug_utils_loader.create_debug_utils_messenger(&debug_messenger_info, None)?
            };

            (Some(debug_utils_loader), Some(debug_messenger))
        } else {
            (None, None)
        };

        Ok(Self {
            entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            enabled_extension_count: extension_names.len(),
            validation_enabled,
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

    pub fn validation_enabled(&self) -> bool {
        self.validation_enabled
    }

    pub fn submit_debug_message(&self, message: &CStr) {
        let Some(debug_utils_loader) = &self.debug_utils_loader else {
            return;
        };

        let message_id_name =
            CString::new("learn-vulkan-renderer.startup").expect("static message id has no nul");
        let callback_data = vk::DebugUtilsMessengerCallbackDataEXT::default()
            .message_id_name(&message_id_name)
            .message_id_number(1)
            .message(message);

        // SAFETY: debug utils loader belongs to this live instance, and callback data points to
        // stack/CString values that remain valid for the duration of this synchronous call.
        unsafe {
            debug_utils_loader.submit_debug_utils_message(
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,
                &callback_data,
            );
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        // SAFETY: `VulkanInstance` 是 `VkInstance` 与 debug messenger 的唯一 owner。
        // 先销毁 debug messenger，再销毁 instance，满足 extension object 的父子关系。
        unsafe {
            if let (Some(debug_utils_loader), Some(debug_messenger)) =
                (&self.debug_utils_loader, self.debug_messenger)
            {
                debug_utils_loader.destroy_debug_utils_messenger(debug_messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

fn instance_layer_available(
    entry: &Entry,
    required_layer: &CStr,
) -> Result<bool, VulkanInstanceError> {
    // SAFETY: 查询 instance layer properties 不依赖已创建的 Vulkan instance。
    let layers = unsafe { entry.enumerate_instance_layer_properties()? };

    Ok(layers.iter().any(|layer| {
        // SAFETY: Vulkan 保证 `layer_name` 是以 nul 结尾的固定长度 C 字符串。
        let layer_name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
        layer_name == required_layer
    }))
}

fn instance_extension_available(
    entry: &Entry,
    required_extension: &CStr,
) -> Result<bool, VulkanInstanceError> {
    // SAFETY: 查询全局 instance extension properties 不依赖已创建的 Vulkan instance。
    let extensions = unsafe { entry.enumerate_instance_extension_properties(None)? };

    Ok(extensions.iter().any(|extension| {
        // SAFETY: Vulkan 保证 `extension_name` 是以 nul 结尾的固定长度 C 字符串。
        let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };
        extension_name == required_extension
    }))
}

fn debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback))
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message = if callback_data.is_null() {
        "<no callback data>"
    } else {
        // SAFETY: Vulkan calls the callback with a valid callback data pointer for this invocation.
        let message_ptr = unsafe { (*callback_data).p_message };

        if message_ptr.is_null() {
            "<no message>"
        } else {
            // SAFETY: Vulkan validation messages are nul-terminated C strings valid for the call.
            unsafe { CStr::from_ptr(message_ptr) }
                .to_str()
                .unwrap_or("<non-utf8 validation message>")
        }
    };

    eprintln!("[Vulkan][{message_severity:?}][{message_types:?}] {message}");

    vk::FALSE
}

pub fn run_instance_shell(config: WindowConfig) -> Result<(), VulkanInstanceError> {
    run_instance_shell_with_label(config, "M1-S3", "learn-vulkan-renderer-m1-s3")
}

pub fn run_validation_shell(config: WindowConfig) -> Result<(), VulkanInstanceError> {
    run_instance_shell_with_label(config, "M1-S4", "learn-vulkan-renderer-m1-s4")
}

fn run_instance_shell_with_label(
    config: WindowConfig,
    milestone_label: &'static str,
    app_name: &'static str,
) -> Result<(), VulkanInstanceError> {
    let event_loop = EventLoop::new()?;
    let mut app = InstanceShell::new(config, milestone_label, app_name);

    event_loop.run_app(&mut app)?;
    app.result
}

#[derive(Debug)]
struct InstanceShell {
    config: WindowConfig,
    milestone_label: &'static str,
    app_name: &'static str,
    vulkan_instance: Option<VulkanInstance>,
    window: Option<Window>,
    result: Result<(), VulkanInstanceError>,
}

impl InstanceShell {
    fn new(config: WindowConfig, milestone_label: &'static str, app_name: &'static str) -> Self {
        Self {
            config,
            milestone_label,
            app_name,
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
        let vulkan_instance = VulkanInstance::new_for_display(display_handle, self.app_name)?;

        println!(
            "{} Vulkan instance created with {} extensions; validation enabled: {}",
            self.milestone_label,
            vulkan_instance.enabled_extension_count(),
            vulkan_instance.validation_enabled()
        );

        let startup_message =
            CString::new(format!("{} debug messenger is ready", self.milestone_label))
                .expect("generated startup message has no nul");
        vulkan_instance.submit_debug_message(&startup_message);

        self.vulkan_instance = Some(vulkan_instance);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: VulkanInstanceError) {
        eprintln!(
            "{} Vulkan instance bootstrap failed: {error}",
            self.milestone_label
        );
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
            println!("{} window close requested", self.milestone_label);
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
