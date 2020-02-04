use std::fs::File;

use anyhow::{anyhow, Result};

use crate::fzf;
use crate::kubeconfig;
use crate::kubectl;
use crate::vars;

pub fn namespace(namespace_name: Option<String>) -> Result<()> {
    vars::ensure_kubie_active()?;
    let namespaces = kubectl::get_namespaces(None)?;

    let enter_namespace = |namespace_name: String| -> Result<()> {
        if !namespaces.contains(&namespace_name) {
            return Err(anyhow!("'{}' is not a valid namespace for the context", namespace_name));
        }

        let mut config = kubeconfig::get_current_config()?;
        config.contexts[0].context.namespace = namespace_name;

        let config_file = File::create(kubeconfig::get_kubeconfig_path()?)?;
        config.write_to(config_file)?;

        Ok(())
    };

    if let Some(namespace_name) = namespace_name {
        enter_namespace(namespace_name)?;
    } else {
        // We only select the context with fzf if stdout is a terminal and if
        // fzf is present on the machine.
        if atty::is(atty::Stream::Stdout) && fzf::is_available() {
            match crate::fzf::select(namespaces.iter())? {
                Some(namespace_name) => {
                    enter_namespace(namespace_name)?;
                }
                None => {
                    println!("Selection cancelled.");
                }
            }
        } else {
            for ns in namespaces {
                println!("{}", ns);
            }
        }
    }

    Ok(())
}