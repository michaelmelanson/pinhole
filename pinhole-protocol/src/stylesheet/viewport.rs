/// The current size of the client's rendering surface, in logical pixels.
///
/// A protocol-level type (rather than depending on a client rendering
/// library's own size type) so `@media`-style rules can be evaluated
/// against it independently of how any particular client obtains it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}
