// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module that lets SWS to run in a "Windows Service" context
//!

use std::ffi::OsString;
use std::thread;
use std::time::Duration;
use std::{env, path::Path};

use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::{helpers, Context, Result, Server, Settings};

const SERVICE_NAME: &str = "static-web-server";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const SERVICE_EXE: &str = "static-web-server.exe";
const SERVICE_DESC: &str =
    "A cross-platform, high-performance and asynchronous web server for static files-serving";
const SERVICE_DISPLAY_NAME: &str = "Static Web Server";

// Generate the Windows Service boilerplate.
// The boilerplate contains the low-level service entry function (ffi_service_main)
// that parses incoming service arguments into Vec<OsString> and passes them to
// user defined service entry (custom_service_main).
define_windows_service!(ffi_service_main, custom_service_main);

fn custom_service_main(_args: Vec<OsString>) {
    if let Err(err) = run_service() {
        tracing::error!("error starting the service: {:?}", err);
    }
}

/// Assigns a particular server state with its properties.
fn set_service_state(
    status_handle: &ServiceStatusHandle,
    current_state: ServiceState,
    checkpoint: u32,
    wait_hint: Duration,
) -> Result {
    let next_status = ServiceStatus {
        // Should match the one from system service registry
        service_type: SERVICE_TYPE,
        // The new state
        current_state,
        // Accept stop events when running
        controls_accepted: ServiceControlAccept::STOP,
        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: ServiceExitCode::Win32(0),
        // Only used for pending states, otherwise must be zero
        checkpoint,
        // Only used for pending states, otherwise must be zero
        wait_hint,
        // Unused for setting status
        process_id: None,
    };

    // Inform the system about the service status
    Ok(status_handle.set_service_status(next_status)?)
}

fn run_service() -> Result {
    // Log is already initialized so there is no need to do it again.
    let opts = Settings::get(false)?;

    tracing::info!("windows service: starting service setup");

    // Create a channel to be able to poll a stop event from the service worker loop.
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());
    let mut shutdown_tx = Some(shutdown_tx);

    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                tracing::debug!("windows service: handled 'ServiceControl::Stop' event");
                if let Some(sender) = shutdown_tx.take() {
                    tracing::debug!("windows service: delegated 'ServiceControl::Stop' event");
                    sender.send(()).unwrap();
                }
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
    tracing::info!("windows service: registering service");

    // Service is running
    set_service_state(
        &status_handle,
        ServiceState::Running,
        1,
        Duration::default(),
    )?;
    tracing::info!("windows service: set service 'Running' state");

    let stop_handler = || {
        // Service is stop pending
        match set_service_state(
            &status_handle,
            ServiceState::StopPending,
            2,
            Duration::from_secs(3),
        ) {
            Ok(()) => tracing::info!("windows service: set service 'StopPending' state"),
            Err(err) => tracing::error!(
                "windows service: error when setting 'StopPending' state: {:?}",
                err
            ),
        }
    };

    // Starting web server
    match Server::new(opts) {
        Ok(server) => {
            if let Err(err) = server.run_as_service(Some(shutdown_rx), stop_handler) {
                tracing::error!(
                    "windows service: error after starting the server: {:?}",
                    err
                );
            }
        }
        Err(err) => {
            tracing::info!("windows service: error starting the server: {:?}", err);
        }
    }

    // Service is stopped
    set_service_state(
        &status_handle,
        ServiceState::Stopped,
        3,
        Duration::from_secs(3),
    )?;
    tracing::info!("windows service: set service 'Stopped' state");

    Ok(())
}

/// Run web server as Windows Server
pub fn run_server_as_service() -> Result {
    // Set current directory to the same as the executable
    let mut path = env::current_exe().unwrap();
    path.pop();
    env::set_current_dir(&path).unwrap();

    // Register generated `ffi_service_main` with the system and start the
    // service, blocking this thread until the service is stopped
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .with_context(|| "error registering generated `ffi_service_main` with the system")?;
    Ok(())
}

/// Install a Windows Service for SWS.
pub fn install_service(config_file: &Path) -> Result {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    // Set the executable path to point the current binary
    let service_binary_path = std::env::current_exe().unwrap().with_file_name(SERVICE_EXE);

    // Set service binary default arguments
    let mut service_binary_arguments = vec![OsString::from("--windows-service=true")];

    // Append a `--config-file` path to the binary arguments if present
    let f = helpers::adjust_canonicalization(config_file);
    if !f.is_empty() {
        service_binary_arguments.push(OsString::from(["--config-file=", &f].concat()));
    }

    // Run the current service as `System` type
    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: service_binary_arguments,
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };

    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(SERVICE_DESC)?;

    println!(
        "Windows Service ({}) is installed successfully!",
        SERVICE_NAME
    );
    println!(
        "Start the service typing: sc.exe start \"{}\" (it requires administrator privileges) or using the 'services.msc' application.",
        SERVICE_NAME
    );

    Ok(())
}

/// Uninstall the current Windows Service for SWS.
pub fn uninstall_service() -> Result {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    service.delete()?;

    println!("Windows Service ({}) is uninstalled!", SERVICE_NAME);

    Ok(())
}
