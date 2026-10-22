//! Opening a tab on a program the user chose: a shell the scan found, or an ssh host.

use gpui::{Context, Window};

use super::Workspace;
use super::layout::Tab;
use crate::settings::{SettingsStore, program_path};
use crate::term::shell::{ShellIntegration, launch_found};
use crate::term::{Launch, Quote, Target, config, opened, spawn};

impl Workspace {
    /// Asks for the new-tab menu to open at `at`, the plus the user pressed.
    pub fn ask_add_menu(&mut self, at: gpui::Point<gpui::Pixels>, cx: &mut Context<Self>) {
        self.add_asked += 1;
        self.add_menu = Some((self.add_asked, at));
        log::info!("app: the plus at {at:?} asks for the new-tab menu");
        cx.notify();
    }

    /// Opens a tab running the shell `found`, in the folder a new session would start in.
    pub fn new_tab_with(
        &mut self,
        found: &crate::term::shell::scan::Found,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(home) = dirs::home_dir() else {
            log::error!("app: no home folder, so no {} session", found.name);
            return;
        };
        let settings = SettingsStore::get(cx).clone();
        let session = self
            .session(self.active)
            .map(|session| session.folder.as_path());
        let folder = settings.start_folder(&home, session);
        let launch = match launch_found(found, folder, &cx.global::<ShellIntegration>().0) {
            Ok(launch) => launch,
            Err(error) => {
                log::error!("app: no {} session: {error:#}", found.name);
                return;
            }
        };
        let (pty, events) = match opened(&launch, (80, 24), (8, 16), config(&settings)) {
            Ok(started) => started,
            Err(failed) => {
                log::error!("app: no {} session:\n{}", found.name, failed.report());
                return;
            }
        };
        let id = self.add(pty, events, launch, window, cx);
        self.tabs.push(Tab::new(id));
        self.active_tab = self.tabs.len() - 1;
        self.select(id, window, cx);
        log::info!("app: tab {} opens as {}", self.active_tab + 1, found.name);
    }

    /// Opens a tab on `host`, running the ssh the system has.
    pub fn connect(&mut self, host: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(home) = dirs::home_dir() else {
            log::error!("app: no home folder, so no ssh to {host}");
            return;
        };
        let ssh = match program_path("ssh") {
            Ok(ssh) => ssh,
            Err(error) => {
                log::error!("app: no ssh to {host}: {error:#}");
                return;
            }
        };
        let settings = SettingsStore::get(cx).clone();
        let launch = Launch {
            program: Some((ssh, vec![host.clone()])),
            cwd: home,
            env: Vec::new(),
            reports: false,
            quote: Quote::Posix,
            target: Target::Local,
        };
        let started = spawn(&launch, (80, 24), (8, 16), config(&settings));
        let (pty, events) = match started {
            Ok(started) => started,
            Err(error) => {
                log::error!("app: no ssh to {host}: {error:#}");
                return;
            }
        };
        let id = self.add(pty, events, launch, window, cx);
        self.tabs.push(Tab::new(id));
        self.active_tab = self.tabs.len() - 1;
        self.select(id, window, cx);
        log::info!("app: ssh connects to {host}");
    }
}
