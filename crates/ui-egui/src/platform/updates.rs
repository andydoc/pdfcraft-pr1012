//! Help ▸ Check for updates and the check at start, through `craft_update`: the flow itself is its
//! `Updater`; this module supplies the suite and the quit path, and shows the result in
//! `craft_update`'s `CheckResultDialog`.
//!
//! This fork (andydoc/pdfcraft-pr1012) differs from upstream in two ways, both from its releases
//! being unsigned pre-releases tagged `X.Y.Z-pr1012[.N]`:
//! - It asks for them itself ([`offered_by_fork`]): `craft_update::check_for_update` only sees
//!   stable versions, and treats a running `-pr1012` copy as a local build.
//! - It offers a download and never "Update and restart" ([`download_prompt`]): `craft_update`
//!   refuses to install anything from a copy that isn't signed.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex, PoisonError};

#[cfg(not(target_arch = "wasm32"))]
use craft_update::updater::{Action, CheckSource, Status, Updater};
#[cfg(not(target_arch = "wasm32"))]
use craft_update::{CheckResultDialog, CheckResultResponse, Release, StartupPrompt, StartupPromptResponse};

use crate::PdfCraftApp;

/// Where every release of this fork is listed.
pub const RELEASES_PAGE: &str = "https://github.com/andydoc/pdfcraft-pr1012/releases";

/// Stands in for the network in tests.
#[cfg(not(target_arch = "wasm32"))]
pub type UpdateSource = CheckSource;

pub(crate) struct Updates {
    #[cfg(not(target_arch = "wasm32"))]
    updater: Updater,
    /// A newer release the check found, waiting for the next frame to become [`Self::prompt`].
    #[cfg(not(target_arch = "wasm32"))]
    found: Arc<Mutex<Option<Release>>>,
    /// The prompt offering a newer release for download.
    #[cfg(not(target_arch = "wasm32"))]
    prompt: Option<StartupPrompt>,
    /// The Updates dialog is showing.
    pub(crate) open: bool,
    /// Ask for a newer release at start, unless Preferences says not to; the desktop app turns
    /// it on, so tests never reach the network.
    #[cfg(not(target_arch = "wasm32"))]
    startup_check: bool,
    #[cfg(not(target_arch = "wasm32"))]
    check_requested: bool,
}

impl Default for Updates {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let found = Arc::default();
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            updater: updater_asking(Arc::new(ask_fork), &found),
            #[cfg(not(target_arch = "wasm32"))]
            found,
            #[cfg(not(target_arch = "wasm32"))]
            prompt: None,
            open: false,
            #[cfg(not(target_arch = "wasm32"))]
            startup_check: false,
            #[cfg(not(target_arch = "wasm32"))]
            check_requested: false,
        }
    }
}

/// An `Updater` that asks `source` and keeps what it finds for [`download_prompt`], so its own
/// prompt, with "Update and restart", never opens.
#[cfg(not(target_arch = "wasm32"))]
fn updater_asking(source: CheckSource, found: &Arc<Mutex<Option<Release>>>) -> Updater {
    let found = found.clone();
    Updater::new(&crate::UPDATE_APP, env!("CARGO_PKG_VERSION")).with_source(Arc::new(move || {
        if let Some(release) = source()? {
            *found.lock().unwrap_or_else(PoisonError::into_inner) = Some(release);
        }
        Ok(None)
    }))
}

/// The release in GitHub's release list `json` to offer a copy running `running`, if any: the
/// newest, pre-releases included, when it is newer by semver (`0.18.2-pr1012` is newer than
/// `0.18.1-pr1012.2`, and `0.18.2-pr1012.1` than `0.18.2-pr1012`).
#[cfg(not(target_arch = "wasm32"))]
pub fn offered_by_fork(json: &str, running: &str) -> Result<Option<Release>, String> {
    let newest = craft_update::parse_github_release_list(json, crate::UPDATE_APP.repo, true)?;
    let running = craft_update::parse_version(running);
    Ok(newest.filter(|release| release.version.as_ref().zip(running.as_ref()).is_some_and(|(new, old)| craft_update::is_newer(new, old))))
}

#[cfg(not(target_arch = "wasm32"))]
fn ask_fork() -> Result<Option<Release>, String> {
    use craft_update::http::{Fetched, Http};
    let app = &crate::UPDATE_APP;
    let http = Http::new(&format!("{}/Update", app.name))?;
    let running = craft_update::self_update::current_version(app, env!("CARGO_PKG_VERSION"));
    let running = running.map_or_else(|| env!("CARGO_PKG_VERSION").to_string(), |v| v.to_string());
    match http.get_api(&format!("https://api.github.com/repos/{}/releases?per_page=30", app.repo), None)? {
        Fetched::Body { text, .. } => offered_by_fork(&text, &running),
        Fetched::RateLimited { .. } => Err("GitHub's request limit was reached".into()),
        Fetched::NotModified | Fetched::NotFound => Ok(None),
    }
}

/// The prompt for a newer `release`: its button opens the release page. The launcher's header
/// installs it; `craft_update` can't, from an unsigned copy.
#[cfg(not(target_arch = "wasm32"))]
pub fn download_prompt(release: Release) -> StartupPrompt {
    let mut prompt = StartupPrompt::new(&crate::UPDATE_APP, release, env!("CARGO_PKG_VERSION"));
    prompt.is_self_updating = false;
    prompt
}

impl PdfCraftApp {
    /// Whether the app asks for a newer release on its first frame (off until the desktop app turns it on).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_startup_update_check(&mut self, on: bool) {
        self.updates.startup_check = on;
    }

    /// Replaces the network with `source` (tests).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_update_source(&mut self, source: UpdateSource) {
        self.updates.updater = updater_asking(source, &self.updates.found);
    }

    /// Help ▸ Check for updates: ask for the latest release and show the outcome.
    pub fn check_for_updates(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.updates.open = true;
            self.updates.check_requested = true;
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.open_url(RELEASES_PAGE);
        }
    }

    /// Starts the checks and picks up their answers (each frame).
    pub(crate) fn poll_updates(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(ctx) = self.ctx.clone() else { return };
            // Settled on the first frame: a preference turned on later waits for the next start.
            let at_start = std::mem::take(&mut self.updates.startup_check) && self.update_check_at_start;
            let updates = &mut self.updates;
            if std::mem::take(&mut updates.check_requested) {
                updates.updater.check_now(&ctx);
            } else if at_start {
                updates.updater.check_at_start(&ctx);
            }
            updates.updater.tick(&ctx);
            let found = updates.found.lock().unwrap_or_else(PoisonError::into_inner).take();
            if let Some(release) = found {
                updates.prompt = Some(download_prompt(release));
                // The prompt answers a requested check, not the Updates dialog.
                updates.open = false;
                updates.updater.dismiss_status();
            }
        }
    }
}

/// The startup prompt, and the Updates dialog that answers Help ▸ Check for updates.
pub(crate) fn dialog(app: &mut PdfCraftApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // The restart helper is scheduled before the app closes, so no update starts over unsaved work.
        let clean = app.first_dirty().is_none();
        match app.updates.updater.show_if(ctx, clean) {
            Action::None => {}
            Action::OpenUrl(url) => app.open_url(&url),
            Action::Close => {
                app.request_quit(ctx);
            }
        }
        if let Some(prompt) = &mut app.updates.prompt {
            match prompt.show(ctx) {
                StartupPromptResponse::Pending => {}
                // `download_prompt` shows no restart button; if one ever answers, it gets the page too.
                StartupPromptResponse::Download | StartupPromptResponse::UpdateAndRestart => {
                    let url = prompt.release.html_url.clone();
                    app.updates.prompt = None;
                    app.open_url(&url);
                }
                StartupPromptResponse::Later => app.updates.prompt = None,
            }
        }
        // A requested check that found a release is answered by the prompt, not by this dialog.
        if app.updates.open && !app.updates.check_requested && matches!(app.updates.updater.status(), Status::Idle) {
            app.updates.open = false;
        }
        if app.updates.open {
            let dialog = CheckResultDialog::new(&crate::UPDATE_APP, env!("CARGO_PKG_VERSION"));
            match dialog.show(ctx, app.updates.updater.status()) {
                CheckResultResponse::None => {}
                CheckResultResponse::Close => {
                    app.updates.open = false;
                    app.updates.updater.dismiss_status();
                }
                CheckResultResponse::CheckAgain => app.updates.check_requested = true,
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}
