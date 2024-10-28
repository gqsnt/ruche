use std::str::FromStr;
use leptos::{component, create_effect, create_signal, provide_context, use_context, view, IntoView, RwSignal, SignalSet, SignalWith};
use leptos_router::{use_params_map, Outlet};
use crate::models::types::PlatformType;

#[component]
pub fn PlatformTypePage()-> impl IntoView{
    let params = use_params_map();
    let platform_type = use_context::<RwSignal<PlatformType>>().expect("PlatformType signal not found");
    create_effect(move |_|{
        let pt = params.with(|params| params.get("platform_type").cloned().unwrap());
        platform_type.set(PlatformType::from_code(pt.as_str()).unwrap());
    });

    view!{
        <Outlet/>
    }
}