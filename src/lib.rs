//! Simple plugin for [nagios]-compatible monitoring systems to check ham-radio repeater status
//! on the [BrandMeister] network.
//!
//! It verifies the last time a ham-radio repeater was seen on the [BrandMeister] network
//! using [BrandMeister]'s API and compares the number of minutes elapsed to thresholds
//! for Warning or Critical state. Tested with [LibreNMS].
//!
//! ## Installation
//! Build the executable and install it in your nagios plugin folder.
//!
//! Example:
//! ```text
//! cargo install check_brandmeister
//! sudo mv $HOME/.cargo/bin /usr/lib/nagios/plugins/
//! ```
//!
//! ## Usage
//!
//! The check_brandmeister plugin is called by Nagios or LibreNMS but can be tested on the command-line.
//!
//! Example:
//! ```text
//! check_brandmeister --repeater 270107
//!
//! BrandMeister repeater 270107 is OK: online status| 'last_seen_min'=0;10;15;;
//! ```
//!
//! ```text
//! USAGE:
//!     check_brandmeister [OPTIONS] --repeater <repeater>
//!
//! OPTIONS:
//!     -c, --critical <critical_minutes>
//!             Inactive time in minutes before Critical state [default: 15]
//!
//!     -h, --help
//!             Print help information
//!
//!     -H, --host <host>
//!             Ignored. For compatibility with nagios Host
//!
//!     -r, --repeater <repeater>
//!             BM repeater id, e.g. 270107
//!
//!     -V, --version
//!             Print version information
//!
//!     -w, --warn <warn_minutes>
//!             Inactive time in minutes before Warning state [default: 10]
//! ```
//!
//! [BrandMeister]: https://brandmeister.network/
//! [nagios]: https://nagios-plugins.org/doc/guidelines.html
//! [LibreNMS]: https://www.librenms.org/

#![warn(missing_docs)]

use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RepeaterStatus {
    last_updated: String,
}

fn get_bm_repeater_last_update(repeater_id: u32) -> Result<String, anyhow::Error> {
    let request_url = format!(
        "http://api.brandmeister.network/v1.0/repeater/?action=get&q={}",
        repeater_id
    );
    let status: RepeaterStatus = ureq::get(&request_url)
        .call()
        .context("error parsing API result, ensure repeater id is valid")?
        .into_json()?;
    Ok(String::from(status.last_updated))
}

/// Return the number of minutes since the repeater was seen online on BrandMeister.
///
/// Example:
/// ```no_run
/// use check_brandmeister::last_seen_minutes;
/// let min = last_seen_minutes("270107");
/// ```
pub fn last_seen_minutes(repeater_id: u32) -> Result<i64> {
    let last_update_str = get_bm_repeater_last_update(repeater_id)?;
    let last_update = Utc.datetime_from_str(&last_update_str, "%Y-%m-%d %H:%M:%S")?;
    Ok(Utc::now().signed_duration_since(last_update).num_minutes())
}
