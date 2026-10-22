//! The health check the window runs on demand, and the report it shows.

use gpui::Context;

use super::Workspace;
use crate::term::shell::{ShellIntegration, Shells};

impl Workspace {
    /// Runs the health check and keeps its report for the window to show. It reads the settings file as a fresh start would, checks the integration scripts are all there, and asks each shell whether it still stands where the scan saw it.
    pub fn check_health(&mut self, cx: &mut Context<Self>) {
        let base = dirs::config_dir()
            .map(|folder| folder.join("ELYTerminal"))
            .unwrap_or_default();
        // The file the store reads at start: the TOML when there is one, else the JSON.
        let config = match base.join("config.toml").exists() {
            true => base.join("config.toml"),
            false => base.join("settings.json"),
        };
        let scripts = cx.global::<ShellIntegration>().0.clone();
        let shells = cx.global::<Shells>().0.clone();
        let report = crate::health::check(&config, &scripts, &shells);
        log::info!("app: health check\n{}", report.text());
        self.health = Some(report);
        cx.notify();
    }
}
