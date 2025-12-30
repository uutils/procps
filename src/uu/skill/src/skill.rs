// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Arg, Command};
#[cfg(unix)]
use nix::{sys::signal, sys::signal::Signal, unistd::Pid};
use uu_snice::{
    collect_pids, construct_verbose_result, print_signals, process_matcher, ActionResult,
};
use uucore::error::{UResult, USimpleError};
#[cfg(unix)]
use uucore::signals::signal_by_name_or_value;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let settings = process_matcher::Settings::try_new(&matches)?;

    // Case0: Print SIGNALS
    if let Some(display) = &settings.display {
        print_signals(display);
        return Ok(());
    }

    // Case1: Send signal
    let take_action = !settings.no_action;
    if let Some(targets) = settings.expressions {
        let pids = collect_pids(&targets);

        #[cfg(unix)]
        let signal_str = matches.get_one::<String>("signal").cloned();

        #[cfg(unix)]
        let signal = if let Some(sig) = signal_str {
            (signal_by_name_or_value(sig.strip_prefix('-').unwrap()).unwrap() as i32).try_into()?
        } else {
            Signal::SIGTERM
        };

        #[cfg(unix)]
        let results = perform_action(&pids, &signal, take_action, settings.interactive);
        #[cfg(not(unix))]
        let results: Vec<Option<ActionResult>> = Vec::new();

        if results.iter().all(|it| it.is_none()) || results.is_empty() {
            return Err(USimpleError::new(1, "no process selection criteria"));
        }

        let error_only = settings.warnings || !settings.verbose;
        if settings.verbose || settings.warnings {
            let output = construct_verbose_result(&pids, &results, error_only, take_action)
                .trim()
                .to_owned();
            println!("{output}");
        } else if !take_action {
            pids.iter().for_each(|pid| println!("{pid}"));
        }
    }

    Ok(())
}

#[cfg(unix)]
fn perform_action(
    pids: &[u32],
    signal: &Signal,
    take_action: bool,
    ask: bool,
) -> Vec<Option<ActionResult>> {
    let sig = if take_action { Some(*signal) } else { None };
    pids.iter()
        .map(|pid| {
            if !ask || uu_snice::ask_user(*pid) {
                Some(match signal::kill(Pid::from_raw(*pid as i32), sig) {
                    Ok(_) => ActionResult::Success,
                    Err(_) => ActionResult::PermissionDenied,
                })
            } else {
                // won't be used, but we need to return (not None)
                Some(ActionResult::Success)
            }
        })
        .collect()
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about("Report processes matching an expression and send a signal to them")
        .override_usage("skill [signal] [options] <expression>")
        .infer_long_args(true)
        .arg_required_else_help(true)
        .arg(Arg::new("signal"))
        .args(uu_snice::clap_args())
}
