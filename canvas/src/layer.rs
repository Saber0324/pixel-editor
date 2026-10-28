use crate::cel::Cel;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayerError {
    #[error("Layer {0} not found in the Sprite")]
    NotFound(String),

    #[error("A Layer with this name already exists")]
    NameCollision,

    #[error("Can't remove the last layer from the Sprite")]
    LastLayer,
}

/// A named layer, shared by every `Frame` of a `Sprite`.
///
/// `cels[f]` is this layer's `Cel` in frame `f`, or `None` if nothing is drawn there.
/// Its length always equals the number of frames in the `Sprite`.
#[derive(Debug, Clone)]
pub struct Layer {
    name: String,
    is_visible: bool,
    pub(crate) cels: Vec<Option<Cel>>,
}

impl Layer {
    /// Creates a new `Layer`.
    pub fn new(name: &str, frame_count: usize) -> Self {
        Self {
            name: name.to_string(),
            is_visible: true,
            cels: vec![None; frame_count],
        }
    }

    /// Returns the name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the name.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Sets the visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    /// Returns the visibility
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Returns the `Cel` within the provided `Frame` index.
    pub fn get_cel(&self, frame: usize) -> Option<&Cel> {
        self.cels.get(frame).and_then(Option::as_ref)
    }
}

impl Default for Layer {
    /// Creates the default "Background" layer with a single empty slot.
    fn default() -> Self {
        Self {
            name: "Background".to_string(),
            is_visible: true,
            cels: vec![None; 1],
        }
    }
}

/// A point in the animation timeline that holds its duration.
#[derive(Debug)]
pub struct Frame {
    duration: Duration,
}

impl Frame {
    /// Creates a `Frame` that stays on screen for the given `Duration`.
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }

    /// Returns the duration of the `Frame`.
    pub fn get_duration(&self) -> Duration {
        self.duration
    }

    /// Sets the duration of the `Frame`.
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }
}

impl Default for Frame {
    /// Returns a `Frame` with a default duration of 100 ms
    fn default() -> Self {
        Self::new(Duration::from_millis(100))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Verifies that layer names can be correctly retrieved and updated,
    /// and that visibility toggles properly modify the layer's state.
    #[test]
    fn test_layer_getters_and_setters() {
        let mut layer = Layer::new("TestLayer", 1);
        assert_eq!(layer.get_name(), "TestLayer");

        assert_eq!(layer.cels, vec![None]);

        layer.set_name("RenamedLayer");
        assert_eq!(layer.get_name(), "RenamedLayer");

        layer.set_visible(false);
        assert!(!layer.is_visible());
    }

    /// Verifies that new frames start with the default duration and that it can be changed.
    #[test]
    fn test_frame_duration() {
        let mut frame = Frame::default();
        assert_eq!(frame.get_duration(), Duration::from_millis(100));

        frame.set_duration(Duration::from_millis(250));
        assert_eq!(frame.get_duration(), Duration::from_millis(250));
    }
}
