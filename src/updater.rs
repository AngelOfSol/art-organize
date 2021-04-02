use self_update::{backends::github, cargo_crate_version, get_target, Status};

pub fn update_app() -> anyhow::Result<Status> {
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("AngelOfSol")
        .repo_name("art-organize")
        .with_target(get_target())
        .build()?
        .fetch()?;

    let status = github::Update::configure()
        .repo_owner("AngelOfSol")
        .repo_name("art-organize")
        .bin_name("art-organize")
        .current_version(cargo_crate_version!())
        .show_output(false)
        .target_version_tag(&releases[0].version)
        .no_confirm(true)
        .build()?
        .update()?;
    Ok(status)
}
pub fn check_for_new_releases() -> anyhow::Result<Option<String>> {
    let mut releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("AngelOfSol")
        .repo_name("art-organize")
        .with_target(get_target())
        .build()?
        .fetch()?;

    Ok(
        self_update::version::bump_is_greater(cargo_crate_version!(), &releases[0].version)?
            .then(|| releases.remove(0).version),
    )
}
