/// Trait for converting into `riven` types.
pub trait IntoRiven<T> {
    fn into_riven(self) -> T;
}

#[cfg(feature = "ssr")]
impl IntoRiven<riven::consts::RegionalRoute> for crate::models::types::RegionType {
    fn into_riven(self) -> riven::consts::RegionalRoute {
        self.to_riven()
    }
}

#[cfg(feature = "ssr")]
impl IntoRiven<riven::consts::PlatformRoute> for crate::models::types::PlatformType {
    fn into_riven(self) -> riven::consts::PlatformRoute {
        self.to_riven()
    }
}