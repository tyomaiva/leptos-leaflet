use leaflet::{LatLng, Map};
use leptos::{html::Div, *};
use wasm_bindgen::prelude::*;
use web_sys::HtmlDivElement;

use leaflet::LocateOptions;

use crate::components::context::provide_leaflet_context;
use crate::components::position::Position;
use crate::{MapEvents, PopupEvents, TooltipEvents};

#[component]
pub fn MapContainer(
    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(into, optional)] style: MaybeSignal<String>,
    /// Centers the map on the given location
    #[prop(into)]
    center: MaybeSignal<Position>,
    /// Zoom level of the map. Defaults to 10.0
    #[prop(optional, default = 10.0)]
    zoom: f64,
    /// Use geolocation from the browser to track the user
    #[prop(optional)]
    locate: bool,
    /// Tracks position of the user on the map
    #[prop(optional)]
    watch: bool,
    /// Enables high-accuracy tracking
    #[prop(optional)]
    enable_high_accuracy: bool,
    /// Sets the view of the map if geolocation is available
    #[prop(optional)]
    set_view: bool,
    #[prop(optional)] map: Option<WriteSignal<Option<Map>>>,
    #[prop(optional)] events: MapEvents,
    #[prop(optional)] popup_events: PopupEvents,
    #[prop(optional)] tooltip_events: TooltipEvents,
    /// An optional node ref for the map `div` container element.
    #[prop(optional)] node_ref: Option<NodeRef<Div>>,
    /// Inner map child nodes
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let map_ref = node_ref.unwrap_or(create_node_ref::<Div>());
    let map_context = provide_leaflet_context();

    let map_load = map_ref;
    map_load.on_load(move |map_div| {
        let html_node = map_div.unchecked_ref::<HtmlDivElement>();
        // Randomize the id of the map
        if html_node.id().is_empty() {
            let id = format!("map-{}", rand::random::<u64>());
            _ = map_div.clone().id(id);
        }
        let events = events.clone();
        let popup_events = popup_events.clone();
        let tooltip_events = tooltip_events.clone();
        _ = map_div.on_mount(move |map_div| {
            let map_div = map_div.unchecked_ref::<HtmlDivElement>();
            let options = leaflet::MapOptions::new();
            options.set_zoom(zoom);
            options.set_center(center.get_untracked().into());
            let leaflet_map = Map::new(&map_div.id(), &options);

            // Setup events
            events.setup(&leaflet_map);
            popup_events.setup(&leaflet_map);
            tooltip_events.setup(&leaflet_map);

            if locate {
                let mut locate_options = LocateOptions::new();
                locate_options.enable_high_accuracy(enable_high_accuracy);
                locate_options.set_view(set_view);
                locate_options.watch(watch);
                leaflet_map.locate_with_options(&locate_options);
            }

            map_context.set_map(&leaflet_map);
            if let Some(map) = map {
                map.set(Some(leaflet_map));
            }
        });
    });

    // We re-center the map on each change of the center
    create_effect(move |_| {
        let new_gps = center.get();
        map_context.map().map(|m| m.fly_to(&LatLng::new(new_gps.lat, new_gps.lng), zoom));
    });

    on_cleanup(move || {
        if let Some(map) = map_context.map_signal().get_untracked() {
            map.remove();
        };
    });

    view! { <div class=move || class.get() node_ref=map_ref style=move || style.get()>{children.map(|child|child())}</div>}
}

#[derive(Debug, Default, Clone)]
pub struct LeafletMap {
    #[cfg(not(feature = "ssr"))]
    pub map: Option<leaflet::Map>,
}

impl LeafletMap {
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "ssr"))]
            map: None,
        }
    }
}
