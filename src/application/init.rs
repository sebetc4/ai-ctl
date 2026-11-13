//! `init`: installs the shipped inventory.

use std::path::Path;

use anyhow::{Result, bail};

use super::ports::{Installed, InventoryStore};

/// Installs the inventory with `models_dir` substituted; refuses to replace an
/// installed one, or a link to one that is gone.
pub fn init(store: &dyn InventoryStore, models_dir: &Path) -> Result<()> {
    match store.install(models_dir)? {
        Installed::Written => Ok(()),
        Installed::AlreadyExists => bail!(
            "{} already exists; edit it, or move it away to install the shipped inventory again",
            store.location().display()
        ),
        Installed::DanglingLink { target } => bail!(
            "{} is a link to {}, which does not exist; restore it, or remove the link to \
             install the shipped inventory",
            store.location().display(),
            target.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use anyhow::anyhow;

    use super::*;
    use crate::domain::inventory::Inventory;

    struct Store {
        outcome: Result<Installed, String>,
        asked_for: RefCell<Option<PathBuf>>,
    }

    impl Store {
        fn new(outcome: Result<Installed, String>) -> Self {
            Self {
                outcome,
                asked_for: RefCell::new(None),
            }
        }
    }

    impl InventoryStore for Store {
        fn location(&self) -> &Path {
            Path::new("/home/user/.config/ai-ctl/inventory.toml")
        }

        fn load(&self) -> Result<Option<Inventory>> {
            unreachable!("init never loads the inventory")
        }

        fn install(&self, models_dir: &Path) -> Result<Installed> {
            *self.asked_for.borrow_mut() = Some(models_dir.to_owned());
            self.outcome.clone().map_err(|message| anyhow!(message))
        }
    }

    #[test]
    fn installs_with_the_given_models_directory() {
        let store = Store::new(Ok(Installed::Written));
        init(&store, Path::new("/data/models")).unwrap();
        assert_eq!(
            store.asked_for.borrow().as_deref(),
            Some(Path::new("/data/models"))
        );
    }

    #[test]
    fn refuses_to_replace_an_installed_inventory() {
        let store = Store::new(Ok(Installed::AlreadyExists));
        let message = init(&store, Path::new("/data/models"))
            .unwrap_err()
            .to_string();
        assert!(
            message.contains("inventory.toml already exists"),
            "{message}"
        );
    }

    #[test]
    fn names_a_link_to_a_missing_inventory_and_its_target() {
        let store = Store::new(Ok(Installed::DanglingLink {
            target: "/data/dotfiles/inventory.toml".into(),
        }));
        let message = init(&store, Path::new("/data/models"))
            .unwrap_err()
            .to_string();
        assert_eq!(
            message,
            "/home/user/.config/ai-ctl/inventory.toml is a link to \
             /data/dotfiles/inventory.toml, which does not exist; restore it, or remove the \
             link to install the shipped inventory"
        );
    }

    #[test]
    fn passes_store_errors_through() {
        let store = Store::new(Err("/data/models is not a directory".into()));
        let message = init(&store, Path::new("/data/models"))
            .unwrap_err()
            .to_string();
        assert_eq!(message, "/data/models is not a directory");
    }
}
