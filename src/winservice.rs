use std::env;
use std::ffi::OsString;
use std::thread;
use std::time::Duration;

use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceDependency,
        ServiceErrorControl, ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState,
        ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::{logger, Context, Result, Server, Settings};

const SERVICE_NAME: &str = "static-web-server";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const SERVICE_EXE: &str = "static-web-server.exe";
const SERVICE_DESC: &str = "A blazing fast and asynchronous web server for static files-serving";
const SERVICE_DISPLAY_NAME: &str = "Static Web Server";

// Generate the windows service boilerplate.
// The boilerplate contains the low-level service entry function (ffi_service_main)
// that parses incoming service arguments into Vec<OsString> and passes them to
// user defined service entry (my_service_main).
define_windows_service!(ffi_service_main, my_service_main);

fn set_service_state(
    status_handle: &ServiceStatusHandle,
    current_state: ServiceState,
) -> Result<()> {
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
        checkpoint: 0,
        // Only used for pending states, otherwise must be zero
        wait_hint: Duration::default(),
        // Unused for setting status
        process_id: None,
    };

    // Tell the system that the service is now running
    Ok(status_handle.set_service_status(next_status)?)
}

fn my_service_main(_args: Vec<OsString>) {
    if let Err(err) = run_service() {
        println!("error starting the service: {:?}", err);
    }
}

fn run_service() -> Result<()> {
    let opts = Settings::get()?;
    logger::init(&opts.general.log_level)?;

    println!("sws service started");

    // Create a channel to be able to poll a stop event from the service worker loop.
    // let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
    println!("registering sws service");

    // Service starts
    set_service_state(&status_handle, ServiceState::StartPending)?;
    println!("sws service start pending");

    match Server::new(Some(shutdown_rx), false) {
        Ok(server) => {
            // Service is running
            set_service_state(&status_handle, ServiceState::Running).unwrap();
            println!("sws service running");

            let r = server.run();
            if r.is_err() {
                println!("error starting the server: {:?}", r.unwrap_err());
            }

            set_service_state(&status_handle, ServiceState::Stopped).unwrap();
            println!("sws service stopping");
        }
        Err(err) => {
            println!("error starting the server: {:?}", err);
            std::process::exit(1);
        }
    }

    set_service_state(&status_handle, ServiceState::Stopped)?;
    println!("sws service stopping");

    Ok(())
}

pub fn run_server_as_service() -> Result<()> {
    // Set current directory to the same as the executable
    let mut path = env::current_exe().unwrap();
    path.pop();
    env::set_current_dir(&path).unwrap();

    // Register generated `ffi_service_main` with the system and start the
    // service, blocking this thread until the service is stopped
    service_dispatcher::start(&SERVICE_NAME, ffi_service_main)
        .with_context(|| "error registering generated `ffi_service_main` with the system")?;
    Ok(())
}

/// Install a Windows Service for SWS.
pub fn install_service() -> Result {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    // Set the executable path to point the current binary
    let service_binary_path = std::env::current_exe().unwrap().with_file_name(SERVICE_EXE);

    // Run the service as `System`
    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![OsString::from("--as-windows-service=true")],
        dependencies: vec![ServiceDependency::from_system_identifier("+network")],
        account_name: None, // run as System
        account_password: None,
    };

    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(SERVICE_DESC)?;

    println!(
        "Windows service ({}) is installed successfully!",
        SERVICE_DISPLAY_NAME
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

    println!(
        "Windows service ({}) is uninstalled successfully!",
        SERVICE_DISPLAY_NAME
    );

    Ok(())
}
