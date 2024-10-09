use core::time::Duration;

use crate::components::context::extend_context_with_overlay;
use crate::components::position::Position;
use leptos::*;
use wasm_bindgen::JsCast;

use crate::components::context::LeafletMapContext;
use crate::core::LeafletMaybeSignal;
use crate::{
    setup_layer_leaflet_option, setup_layer_leaflet_option_ref, DragEvents, LayerEvents,
    MouseEvents, MoveEvents, PopupEvents, TooltipEvents,
};

#[component(transparent)]
pub fn Marker(
    /// Position for the Marker
    #[prop(into)]
    position: MaybeSignal<Position>,
    #[prop(into)]
    feedback_position: WriteSignal<Option<Position>>,
    #[prop(into, optional)] draggable: MaybeSignal<bool>,
    #[prop(into, optional)] keyboard: LeafletMaybeSignal<bool>,
    #[prop(into, optional)] title: LeafletMaybeSignal<String>,
    #[prop(into, optional)] alt: LeafletMaybeSignal<String>,
    #[prop(into, optional)] interactive: LeafletMaybeSignal<bool>,
    #[prop(into, optional)] z_index_offset: LeafletMaybeSignal<f64>,
    #[prop(into, optional)] opacity: LeafletMaybeSignal<f64>,
    #[prop(into, optional)] rise_on_hover: LeafletMaybeSignal<bool>,
    #[prop(into, optional)] rise_offset: LeafletMaybeSignal<f64>,
    #[prop(into, optional)] pane: LeafletMaybeSignal<String>,
    #[prop(into, optional)] shadow_pane: LeafletMaybeSignal<String>,
    #[prop(into, optional)] bubbling_mouse_events: LeafletMaybeSignal<bool>,
    #[prop(into, optional)] auto_pan: LeafletMaybeSignal<bool>,
    #[prop(into, optional)] auto_pan_padding: LeafletMaybeSignal<(f64, f64)>,
    #[prop(into, optional)] auto_pan_speed: LeafletMaybeSignal<f64>,
    #[prop(into, optional)] icon_class: LeafletMaybeSignal<String>,
    #[prop(into, optional)] icon_url: LeafletMaybeSignal<String>,
    #[prop(into, optional)] icon_size: LeafletMaybeSignal<(f64, f64)>,
    #[prop(into, optional)] icon_anchor: LeafletMaybeSignal<(f64, f64)>,
    #[prop(into, optional)] attribution: LeafletMaybeSignal<String>,
    #[prop(into, optional)] rotation: LeafletMaybeSignal<f64>,
    #[prop(into, optional)] move_events: MoveEvents,
    #[prop(into, optional)] mouse_events: MouseEvents,
    #[prop(into, optional)] drag_events: DragEvents,
    #[prop(into, optional)] layer_events: LayerEvents,
    #[prop(into, optional)] popup_events: PopupEvents,
    #[prop(into, optional)] tooltip_events: TooltipEvents,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let position_tracking = position;
    let map_context = use_context::<LeafletMapContext>().expect("Map context not found");

    let overlay_context = extend_context_with_overlay();
    let overlay = store_value(None::<leaflet::Marker>);

    create_effect(move |_| {
        if let Some(map) = map_context.map() {
            let options = leaflet::MarkerOptions::new();
            let drag = draggable.get_untracked();
            if drag {
                options.set_draggable(drag);
            }
            setup_layer_leaflet_option!(keyboard, options);
            setup_layer_leaflet_option_ref!(title, options);
            setup_layer_leaflet_option_ref!(alt, options);
            setup_layer_leaflet_option!(interactive, options);
            setup_layer_leaflet_option!(z_index_offset, options);
            setup_layer_leaflet_option!(opacity, options);
            setup_layer_leaflet_option!(rise_on_hover, options);
            setup_layer_leaflet_option!(rise_offset, options);
            setup_layer_leaflet_option_ref!(pane, options);
            setup_layer_leaflet_option_ref!(shadow_pane, options);
            setup_layer_leaflet_option!(bubbling_mouse_events, options);
            setup_layer_leaflet_option!(auto_pan, options);
            setup_layer_leaflet_option!(auto_pan_speed, options);
            setup_layer_leaflet_option_ref!(attribution, options);

            if let Some((x, y)) = auto_pan_padding.get_untracked() {
                options.set_auto_pan_padding(leaflet::Point::new(x, y));
            }
            if let Some(icon_url) = icon_url.get_untracked() {
                let icon_options = leaflet::IconOptions::new();
                icon_options.set_icon_url(icon_url);
                if let Some((x, y)) = icon_size.get_untracked() {
                    icon_options.set_icon_size(leaflet::Point::new(x, y));
                }
                if let Some((x, y)) = icon_anchor.get_untracked() {
                    icon_options.set_icon_anchor(leaflet::Point::new(x, y));
                }
                let icon = leaflet::Icon::new(&icon_options);
                options.set_icon(icon);
            } else if let Some(icon_class) = icon_class.get_untracked() {
                let icon_options = leaflet::DivIconOptions::new();
                icon_options.set_class_name(icon_class);
                if let Some((x, y)) = icon_size.get_untracked() {
                    icon_options.set_icon_size(leaflet::Point::new(x, y));
                }
                if let Some((x, y)) = icon_anchor.get_untracked() {
                    icon_options.set_icon_anchor(leaflet::Point::new(x, y));
                }
                let icon = leaflet::DivIcon::new(&icon_options);
                options.set_icon(icon.into());
            }
            let marker =
                leaflet::Marker::new_with_options(&position.get_untracked().into(), &options);

            mouse_events.setup(&marker);
            move_events.setup(&marker);
            drag_events.setup(&marker);
            popup_events.setup(&marker);
            tooltip_events.setup(&marker);
            layer_events.setup(&marker);

            marker.add_to(&map);
            overlay_context.set_container(&marker);
            overlay.set_value(Some(marker));
        };
    });

    const DRAG_POLL_INTERVAL: Duration = Duration::from_millis(100);
    let _ = leptos_use::use_interval_fn(
        move || {
            if let Some(marker) = overlay.get_value() {
                let gps = marker.get_lat_lng();
                feedback_position.set(Some(Position::new(gps.lat(), gps.lng())));
            }
        },
        DRAG_POLL_INTERVAL.as_millis() as u64,
    );

    let position_stop = watch(
        move || position_tracking.get(),
        move |position_tracking, _, _| {
            if let Some(marker) = overlay.get_value() {
                marker.set_lat_lng(&position_tracking.into());
            }
        },
        false,
    );

    let opacity_stop = watch(
        move || opacity.get(),
        move |opacity, _, _| {
            overlay.get_value().and_then(|marker| {
                opacity.map(|opacity| {
                    marker.set_opacity(opacity);
                })
            });
        },
        false,
    );

    let drag_stop = watch(
        move || draggable.get(),
        move |&draggable, _, _| {
            if let Some(marker) = overlay.get_value() {
                match draggable {
                    true => marker.dragging().enable(),
                    false => marker.dragging().disable(),
                };
            }
        },
        false,
    );

    let rotation_stop = watch(
        move || rotation.get(),
        move |&rotation, prev_rotation, _| {
            if let (Some(marker), Some(rotation)) = (overlay.get_value(), rotation) {
                if Some(rotation.trunc()) == prev_rotation.copied().flatten().map(|r| r.trunc()) {
                    return;
                }
                if let Ok(internal_icon) = js_sys::Reflect::get(&marker, &"_icon".into()) {
                    let internal_icon = internal_icon.unchecked_ref::<web_sys::HtmlElement>();
                    _ = internal_icon
                        .style()
                        .set_property("--gps_rotation", &format!("{}deg", rotation));

                    let transform = internal_icon
                        .style()
                        .get_property_value("transform")
                        .unwrap_or_default();

                    let transform = match transform.contains("rotate(") {
                        true => transform
                            .split_whitespace()
                            .map(|part| match part.starts_with("rotate(") {
                                true => format!("rotate({}deg)", rotation),
                                false => part.to_string(),
                            })
                            .collect::<Vec<String>>()
                            .join(" "),
                        false => format!("{} rotate({}deg)", transform, rotation),
                    };

                    let _ = internal_icon.style().set_property("transform", &transform);
                    let _ = internal_icon
                        .style()
                        .set_property("transform-origin", "center");
                }
            }
        },
        false,
    );

    on_cleanup(move || {
        position_stop();
        opacity_stop();
        drag_stop();
        rotation_stop();
        if let Some(overlay) = overlay.get_value() {
            overlay.remove();
        }
    });

    children.map(|child| child())
}
