use std::error::Error;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, Document, HtmlCanvasElement, HtmlElement, Request, RequestInit, RequestMode, Response,
};
use wgpu::{
    Adapter, Device, DeviceDescriptor, Features, Instance, Limits, MemoryHints, PowerPreference,
    Queue, Surface, SurfaceTarget,
};

pub struct WgpuApp<'a> {
    instance: Instance,
    surface: Box<Surface<'a>>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl<'a> WgpuApp<'a> {
    pub async fn new(document: Document, parent_element: HtmlElement) -> Result<Self, JsValue> {
        // Create a canvas and try with a WebGPU backend.
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        parent_element.append_child(&canvas)?;

        match Self::with_webgpu_backend(canvas.clone()).await {
            Ok(app) => return Ok(app),
            Err(_) => (),
        };

        // If WebGPU backend fails, destroy the canvas and try with a WebGL backend.
        parent_element.remove_child(&canvas)?;
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        parent_element.append_child(&canvas)?;

        match Self::with_webgl_backend(canvas).await {
            Ok(app) => return Ok(app),
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };
    }

    pub async fn with_webgpu_backend(canvas: HtmlCanvasElement) -> Result<Self, Box<dyn Error>> {
        let instance_desc = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            flags: wgpu::InstanceFlags::empty(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        };

        let instance = wgpu::Instance::new(instance_desc);

        let surface_target: SurfaceTarget = SurfaceTarget::Canvas(canvas);
        let surface = match instance.create_surface(surface_target) {
            Ok(surface) => surface,
            Err(e) => {
                console::error_1(&e.to_string().into());
                return Err(Box::new(e));
            }
        };

        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        let adapter = match instance.request_adapter(&request_adapter_options).await {
            Some(adapter) => adapter,
            None => {
                console::error_1(&"No suitable adapter found for WebGPU backend".into());
                return Err("No suitable adapter found for WebGPU backend".into());
            }
        };

        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            memory_hints: MemoryHints::MemoryUsage,
        };

        let (device, queue) = match adapter.request_device(&device_descriptor, None).await {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                console::error_1(&e.to_string().into());
                return Err(Box::new(e));
            }
        };

        console::log_1(&"WebGPU backend initialized".into());

        Ok(Self {
            instance,
            surface: Box::new(surface),
            adapter,
            device,
            queue,
        })
    }

    pub async fn with_webgl_backend(canvas: HtmlCanvasElement) -> Result<Self, Box<dyn Error>> {
        let instance_desc = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::empty(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        };

        let instance = wgpu::Instance::new(instance_desc);

        let surface_target: SurfaceTarget = SurfaceTarget::Canvas(canvas);
        let surface = match instance.create_surface(surface_target) {
            Ok(surface) => surface,
            Err(e) => {
                console::error_1(&e.to_string().into());
                return Err(Box::new(e));
            }
        };

        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        let adapter = match instance.request_adapter(&request_adapter_options).await {
            Some(adapter) => adapter,
            None => {
                console::error_1(&"No suitable adapter found for WebGL backend".into());
                return Err("No suitable adapter found for WebGL backend".into());
            }
        };

        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            memory_hints: MemoryHints::MemoryUsage,
        };

        let (device, queue) = match adapter.request_device(&device_descriptor, None).await {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                console::error_1(&e.to_string().into());
                return Err(Box::new(e));
            }
        };

        console::log_1(&"WebGL backend initialized".into());

        Ok(Self {
            instance,
            surface: Box::new(surface),
            adapter,
            device,
            queue,
        })
    }
}

async fn fetch(url: &str) -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.text()?).await?;

    // Send the JSON response back to JS.
    Ok(json)
}

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    // Access the window, document, and body elements
    let window = web_sys::window().expect("No global `window` exists");
    let document = window.document().expect("Should have a document on window");
    let body = document.body().expect("Document should have a body");

    // Create a new WgpuApp
    let app = WgpuApp::new(document, body).await?;

    // Fetch shader.wgsl file
    let shader_url = "shader.wgsl";
    let shader = fetch(shader_url).await?;
    console::log_1(&shader);

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
