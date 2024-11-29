use crate::views::platform_type_page::PlatformTypePage;
use crate::views::summoner_page::SummonerPage;
use leptos::config::LeptosOptions;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};

use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, Title};
use leptos_router::components::{ParentRoute, Redirect};
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
};

pub const SITE_URL: &str = "https://next-level.xyz";

#[derive(Clone, reactive_stores_macro::Store, Serialize, Deserialize, Archive, Default)]
pub struct MetaStore {
    pub title: String,
    pub description: String,
    pub image: String,
    pub url: String,
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let meta_store = reactive_stores::Store::new(MetaStore::default());
    provide_context(meta_store);

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/broken-gg.css" />
        <Link rel="preload" as_="image" href="/assets/champions.avif" />
        <Link rel="preload" as_="image" href="/assets/items.avif" />
        <Link rel="preload" as_="image" href="/assets/summoner_spells.avif" />
        <Link rel="preload" as_="image" href="/assets/perks.avif" />
        // sets the document title
        <Title text=move || meta_store.title().get() />
        <Meta name="color-scheme" content="dark light" />
        <Meta name="og:type" content="website" />
        <Meta name="og:site_name" content="Broken.gg" />
        <Meta name="robots" content="index,follow" />

        <Meta name="description" content=move || meta_store.description().get() />
        <Meta name="og:title" content=move || meta_store.title().get() />
        <Meta name="og:description" content=move || meta_store.description().get() />
        <Meta name="og:image" content=move || meta_store.image().get() />
        <Meta name="og:url" content=move || meta_store.url().get() />
        <Link rel="canonical" prop:href=move || format!("{}{}", SITE_URL, meta_store.url().get()) />
        <Link rel="icon" type_="image/x-icon" href="/assets/favicon.ico" />

        // content for this welcome page
        <Router>
            <main class="bg-gray-900 flex items-start justify-center min-h-screen w-full text-gray-200">
                <Routes transition=true fallback=|| view!{<div class="text-center">Page Not Found</div>}>
                    <Route path=StaticSegment("") view=move || view! { <Redirect path="platform/EUW" /> } />
                    <ParentRoute path=(StaticSegment("platform"), ParamSegment("platform_type")) view=PlatformTypePage>
                        <Route path=StaticSegment("") view=move || view! { }/>
                        <Route
                            path=(StaticSegment("summoners"), ParamSegment("summoner_slug"))
                            view=SummonerPage
                        />
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}
