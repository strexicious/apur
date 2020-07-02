use super::renderable::Renderable;

// not finalised API, but this is the main trait
// that should be implemented to make renderers
pub trait RenderingTechnique {
    // may want encoder, device, swapchain things like these as well
    fn render(renderables: Vec<Renderable>);
}
