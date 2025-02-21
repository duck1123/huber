use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use hubcaps::{Credentials, Github};
use log::{debug, info};

use huber_common::file::is_empty_dir;
use huber_common::model::package::Package;
use huber_common::model::release::Release;
use huber_common::result::Result;

#[async_trait]
pub(crate) trait GithubClientTrait {
    async fn get_latest_release(&self, owner: &str, repo: &str, pkg: &Package) -> Result<Release>;
    async fn get_release(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
        pkg: &Package,
    ) -> Result<Release>;
    async fn get_releases(&self, owner: &str, repo: &str, pkg: &Package) -> Result<Vec<Release>>;
    async fn download_artifacts<P: AsRef<Path> + Send>(
        &self,
        release: &Release,
        dir: P,
    ) -> Result<()>;
    async fn clone<P: AsRef<Path> + Send + Sync>(
        &self,
        owner: &str,
        repo: &str,
        dir: P,
    ) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct GithubClient {
    github: Github,
    github_key: Option<PathBuf>,
}

unsafe impl Send for GithubClient {}

unsafe impl Sync for GithubClient {}

impl GithubClient {
    pub(crate) fn new(
        github_credentials: Option<Credentials>,
        github_key: Option<PathBuf>,
    ) -> Self {
        Self {
            github: Github::new("huber", github_credentials).unwrap(),
            github_key,
        }
    }

    fn clone_fresh<P: AsRef<Path> + Send>(&self, url: &str, dir: P) -> Result<Repository> {
        if let Some(key) = self.github_key.as_ref() {
            if key.exists() {
                info!("Cloning huber repo via SSH");

                // Prepare builder.
                let fetch_options = self.create_git_fetch_options(key.clone())?;
                let mut builder = git2::build::RepoBuilder::new();
                builder.fetch_options(fetch_options);

                return Ok(builder.clone(&url, dir.as_ref())?);
            } else {
                info!("The configured github key not found, {:?}", key);
            }
        }

        info!("Cloning huber repo via https");
        //Note: if encountering authentication required, probably hit this issue https://github.com/rust-lang/git2-rs/issues/463
        Ok(Repository::clone(&url, &dir)?)
    }

    fn fetch_merge_repo<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        debug!("Merging huber repo update");

        let mut fetch_options = if let Some(key) = self.github_key.as_ref() {
            if key.exists() {
                info!("Fetching huber repo via SSH");
                Some(self.create_git_fetch_options(key.clone())?)
            } else {
                None
            }
        } else {
            None
        };

        let repo = Repository::open(&dir)?;

        // fetch the origin
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&["master"], fetch_options.as_mut(), None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let commit = repo.reference_to_annotated_commit(&fetch_head)?;

        // merge local, and checkout
        let reference_name = format!("refs/heads/{}", "master");
        let mut reference = repo.find_reference(&reference_name)?;
        let name = reference.name().expect("");
        repo.set_head(name)?;
        reference.set_target(commit.id(), "")?;

        Ok(repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?)
    }

    fn create_git_fetch_options<T: AsRef<Path> + 'static>(&self, key: T) -> Result<FetchOptions> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            Cred::ssh_key(username_from_url.unwrap(), None, key.as_ref(), None)
        });

        // Prepare fetch options.
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        Ok(fo)
    }
}

#[async_trait]
impl GithubClientTrait for GithubClient {
    async fn get_latest_release(&self, owner: &str, repo: &str, pkg: &Package) -> Result<Release> {
        debug!("Getting the latest release of package {}", &pkg);

        let release = self.github.repo(owner, repo).releases().latest().await?;
        let mut release = Release::from(release);

        release.name = pkg.name.clone();
        release.package.name = pkg.name.clone();
        release.package.source = pkg.source.clone();
        release.package.targets = pkg.targets.clone();
        release.package.version = Some(release.version.clone());

        Ok(release)
    }

    async fn get_release(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
        pkg: &Package,
    ) -> Result<Release> {
        debug!("Getting the specific release of package {}/{}", &pkg, tag);

        let release = self.github.repo(owner, repo).releases().by_tag(tag).await?;
        let mut release = Release::from(release);

        release.name = pkg.name.clone();
        release.package.name = pkg.name.clone();
        release.package.source = pkg.source.clone();
        release.package.targets = pkg.targets.clone();
        release.package.version = Some(release.version.clone());

        Ok(release)
    }

    async fn get_releases(&self, owner: &str, repo: &str, pkg: &Package) -> Result<Vec<Release>> {
        debug!("Getting all releases of package {}", &pkg);

        let releases = self.github.repo(owner, repo).releases().list().await?;
        let releases = releases
            .into_iter()
            .map(|it| {
                let mut release = Release::from(it);

                release.name = pkg.name.clone();
                release.package.name = pkg.name.clone();
                release.package.source = pkg.source.clone();
                release.package.targets = pkg.targets.clone();
                release.package.release_kind = release.kind.clone();

                release
            })
            .collect();

        Ok(releases)
    }

    async fn download_artifacts<P: AsRef<Path> + Send>(
        &self,
        _release: &Release,
        _dir: P,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn clone<P: AsRef<Path> + Send + Sync>(
        &self,
        owner: &str,
        repo: &str,
        dir: P,
    ) -> Result<()> {
        info!("Cloning huber github repo");

        let url = format!("https://github.com/{}/{}", owner, repo);

        if is_empty_dir(&dir) {
            self.clone_fresh(&url, &dir)?;
            return Ok(());
        }

        if let Err(e) = self.fetch_merge_repo(&dir) {
            debug!("Failed to fetch huber github repo, {:?}", e);

            let _ = remove_dir_all(&dir);
            self.clone_fresh(&url, &dir)?;
        }

        Ok(())
    }
}
