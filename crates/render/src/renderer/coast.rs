//! Cross-fade paired coastline coverage before compositing it onto the page.
use super::{Arc, Frame, Node, NodeKind, RenderContext, Renderer, group};
use vectorcraft_color::{BlendMode, Paint};
use vectorcraft_doc::Knockout;

impl Renderer {
    pub(super) fn draw_map_layers(&mut self, ctx: &mut RenderContext, f: &Frame, mut layers: &[Arc<Node>]) {
        while let Some((first, rest)) = layers.split_first() {
            if let Some((second, after)) = rest.split_first()
                && self.draw_coast_pair(ctx, f, first, second)
            {
                layers = after;
            } else {
                self.draw_arc(ctx, f, first);
                layers = rest;
            }
        }
    }

    fn draw_coast_pair(&mut self, ctx: &mut RenderContext, f: &Frame, first: &Arc<Node>, second: &Arc<Node>) -> bool {
        let (Some(zoom), Some(a), Some(b)) = (f.opts.world_map_zoom, first.extras().zoom_fade, second.extras().zoom_fade) else {
            return false;
        };
        let filter = f.doc.active_map_filter();
        if [first, second].into_iter().any(|layer| {
            !layer.extras().detail_level || !layer.visible || f.opts.hidden.contains(&layer.id) || filter.is_some_and(|filter| filter.hides(layer))
        }) {
            return false;
        }
        let (coarse, fine, outgoing, incoming) = if a.fade_out.is_some() && a.fade_out == b.fade_in {
            (first, second, a, b)
        } else if b.fade_out.is_some() && b.fade_out == a.fade_in {
            (second, first, b, a)
        } else {
            return false;
        };
        let weight = incoming.opacity(zoom);
        // A middle detail level can have both intervals. Do not pair it while
        // its other interval (or the outgoing level's own fade-in) contributes.
        if weight <= 0.0 || weight >= 1.0 || (outgoing.opacity(zoom) + weight - 1.0).abs() > f32::EPSILON {
            return false;
        }
        let bounds = vectorcraft_geom::union_opt(self.bounds_of(f.symbols, coarse), self.bounds_of(f.symbols, fine));
        let Some((region, sub)) = group::area(ctx, f, bounds) else { return false };
        // Two temporary pixmaps only, bounded even for unusually large canvases.
        if u64::from(region.w) * u64::from(region.h) > 16_000_000 {
            return false;
        }
        let (Some(coarse_fill), Some(fine_fill)) = (coast_art(coarse, true), coast_art(fine, true)) else { return false };
        let dominant = if weight < 0.5 { coarse } else { fine };
        let Some(strokes) = coast_art(dominant, false) else { return false };
        let mut fill = self.offscreen(region, &sub, None, &mut |r, ctx, f| r.draw_arc(ctx, f, &coarse_fill));
        let next = self.offscreen(region, &sub, None, &mut |r, ctx, f| r.draw_arc(ctx, f, &fine_fill));
        // vello stores premultiplied RGBA. Interpolating alpha and premultiplied
        // channels together preserves opaque coverage where the silhouettes overlap.
        let mix = |a: u8, b: u8| (f32::from(a) * (1.0 - weight) + f32::from(b) * weight).round() as u8;
        for (a, b) in fill.data_mut().iter_mut().zip(next.data()) {
            (a.r, a.g, a.b, a.a) = (mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), mix(a.a, b.a));
        }
        group::draw_pixmap(ctx, Arc::new(fill), (f64::from(region.x), f64::from(region.y)));
        self.draw_arc(ctx, f, &strokes);
        true
    }
}

/// Split a plain coastline layer into a transient fill or stroke pass. Complex
/// appearances and mixed content fall back to the ordinary layer fade.
fn coast_art(layer: &Arc<Node>, fills: bool) -> Option<Arc<Node>> {
    let mut count = 0;
    let mut art = split(layer, fills, true, &mut count)?;
    if count == 0 {
        return None;
    }
    art.extras_mut().zoom_fade = None;
    Some(Arc::new(art))
}

fn split(node: &Node, fills: bool, root: bool, count: &mut usize) -> Option<Node> {
    if node.blend != BlendMode::Normal
        || node.opacity != 1.0
        || node.knockout == Knockout::On
        || node.extras().mask.is_some()
        || node.extras().trace.is_some()
        || !node.appearance.effects.is_empty()
        || (!root && node.extras().zoom_fade.is_some())
    {
        return None;
    }
    let mut art = node.clone();
    match &mut art.kind {
        NodeKind::Layer { clip: false, template: false, preview: true, children, .. } | NodeKind::Group { clip: false, children }
            if node.appearance.items.is_empty() =>
        {
            *children = children.iter().map(|c| split(c, fills, false, count).map(Arc::new)).collect::<Option<_>>()?;
        }
        NodeKind::Path { guide: false, clipping: false, .. } | NodeKind::Compound { .. }
            if node.extras().geography.as_ref().is_some_and(|geo| geo.categories.iter().any(|c| c == "core:coastline"))
                && node.appearance.items.iter().all(|i| {
                    i.effects().is_empty()
                        && i.blend() == BlendMode::Normal
                        && !i.overprint()
                        && matches!(i.paint(), Paint::None | Paint::Solid { .. })
                }) =>
        {
            *count += 1;
            art.appearance.items.retain(|item| item.is_fill() == fills);
        }
        _ => return None,
    }
    Some(art)
}
