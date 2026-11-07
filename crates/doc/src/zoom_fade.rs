//! Optional, view-dependent opacity for geographic detail layers.
//! Vector geometry and the stored layer opacity remain unchanged.
use serde::{Deserialize, Deserializer, Serialize};

/// Zoom is screen points per document point (not the displayed percent).
/// Outside a fade-in interval the layer is hidden below it and fully shown above it;
/// outside a fade-out interval the layer is fully shown below it and hidden above it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ZoomFade {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<[f64; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ZoomFadeWire {
    #[serde(default)]
    fade_in: Option<[f64; 2]>,
    #[serde(default)]
    fade_out: Option<[f64; 2]>,
}

impl<'de> Deserialize<'de> for ZoomFade {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ZoomFadeWire::deserialize(deserializer)?;
        Self::new(wire.fade_in, wire.fade_out).map_err(serde::de::Error::custom)
    }
}

impl ZoomFade {
    /// Reject non-finite, non-positive and reversed intervals before they enter a document.
    pub fn new(fade_in: Option<[f64; 2]>, fade_out: Option<[f64; 2]>) -> Result<Self, &'static str> {
        for range in [fade_in, fade_out].into_iter().flatten() {
            if !(range[0].is_finite() && range[1].is_finite() && range[0] > 0.0 && range[1] > range[0]) {
                return Err("zoom fade ranges require finite, positive start < end");
            }
        }
        Ok(Self { fade_in, fade_out })
    }

    /// A smoothstep transition avoids a visible sudden change of fading speed at either end.
    pub fn opacity(self, zoom: f64) -> f32 {
        if !(zoom.is_finite() && zoom > 0.0) {
            return 0.0;
        }
        let ramp = |[start, end]: [f64; 2]| {
            let t = ((zoom - start) / (end - start)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let fade_in = self.fade_in.map_or(1.0, ramp);
        let fade_out = self.fade_out.map_or(1.0, |[start, end]| 1.0 - ramp([start, end]));
        (fade_in * fade_out).clamp(0.0, 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complementary_coastline_fades_are_smooth() {
        let coarse = ZoomFade::new(None, Some([200.0, 500.0])).unwrap();
        let fine = ZoomFade::new(Some([200.0, 500.0]), None).unwrap();
        for zoom in [100.0, 200.0, 250.0, 350.0, 450.0, 500.0, 1000.0] {
            assert!((coarse.opacity(zoom) + fine.opacity(zoom) - 1.0).abs() < 1e-6);
        }
        assert_eq!(coarse.opacity(100.0), 1.0);
        assert_eq!(coarse.opacity(500.0), 0.0);
        assert_eq!(fine.opacity(100.0), 0.0);
        assert_eq!(fine.opacity(500.0), 1.0);
    }

    #[test]
    fn fade_roundtrip_and_invalid_ranges() {
        let fade = ZoomFade::new(Some([10.0, 20.0]), Some([200.0, 500.0])).unwrap();
        let value = serde_json::to_value(fade).unwrap();
        let loaded: ZoomFade = serde_json::from_value(value).unwrap();
        assert_eq!(loaded, fade);
        for range in [[0.0, 2.0], [2.0, 2.0], [3.0, 2.0], [f64::NAN, 2.0], [1.0, f64::INFINITY]] {
            assert!(ZoomFade::new(Some(range), None).is_err());
        }
        for bad in [serde_json::json!({"fade_in": [200, 100]}), serde_json::json!({"fade_out": [-1, 10]}), serde_json::json!({"fade_out": [1, 1]})] {
            assert!(serde_json::from_value::<ZoomFade>(bad).is_err());
        }
    }
}
