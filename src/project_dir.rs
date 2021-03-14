use directories::ProjectDirs;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROJECT: ProjectDirs =
        ProjectDirs::from("com", "aos-studios", "ArtOrganize").unwrap();
}
