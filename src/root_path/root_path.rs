use bevy::prelude::Resource;

#[derive(Resource)]
pub struct RootPath {
    pub path: String,
}

impl RootPath {
    pub fn from(path: &str) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
}
