use wasm_bindgen::prelude::*;
use web_sys::console;
use wgpu::SurfaceTarget;

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    // Access the window, document, and body elements
    let window = web_sys::window().expect("No global `window` exists");
    let document = window.document().expect("Should have a document on window");
    let body = document.body().expect("Document should have a body");

    // Create an InstanceDescriptor with the desired backends and flags
    let instance_desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL,
        flags: wgpu::InstanceFlags::empty(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        gles_minor_version: wgpu::Gles3MinorVersion::default(),
    };

    // Create the Instance with the specified backends and options
    let instance = wgpu::Instance::new(instance_desc);

    // Add a canvas element to the document
    let c = document.create_element("canvas")?;
    let canvas = c
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    body.append_child(&canvas)?;

    let surface_target: SurfaceTarget = SurfaceTarget::Canvas(canvas);
    let surface = match instance.create_surface(surface_target) {
        Ok(surface) => surface,
        Err(e) => {
            console::error_1(&e.to_string().into());
            return Ok(());
        }
    };

    // Request an adapter that matches the given options
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await;

    // Handle the case where no adapter is found
    let adapter = match adapter {
        Some(adapter) => adapter,
        None => {
            let val = document.create_element("p")?;
            val.set_inner_html("No suitable adapter found");
            body.append_child(&val)?;
            return Ok(());
        }
    };

    // Get information about the selected adapter
    let adapter_info = adapter.get_info();

    // Determine the backend that was selected
    let backend_str = match adapter_info.backend {
        wgpu::Backend::BrowserWebGpu => "WebGPU",
        wgpu::Backend::Gl => "WebGL",
        _ => "Other",
    };

    // Create a new paragraph element and set its content to the selected backend
    let val = document.create_element("p")?;
    val.set_inner_html(&format!("Selected backend: {}", backend_str));

    // Append the paragraph to the body of the document
    body.append_child(&val)?;

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
