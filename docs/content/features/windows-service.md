# Windows Service

**`SWS`** can be also executed as a [Windows Service](https://docs.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc783643(v=ws.10)).

This feature is disabled by default and can be controlled by the boolean `-s, --windows-service` option or the equivalent [SERVER_WINDOWS_SERVICE](./../configuration/environment-variables.md#server_windows_service) env.

![Static Web Server running as a Windows Service](https://user-images.githubusercontent.com/1700322/169807572-d62a7bab-b596-4597-85f7-31a7c02aeefe.png)
> _Static Web Server running as a Windows Service and displayed by 'services.msc' application._

**Important notes**

- This is a Windows platform-specific feature. It means the `--windows-service` argument and its env will not be present in Unix-like systems.
- Running SWS as a Windows service doesn't require enabling it via the [configuration file](../configuration/config-file.md) (`windows-service = true`) because it's already enabled during the service installation.

## Service privileges

To either install or uninstall the SWS Windows service requires *administrator* privileges, so make sure to open the terminal application as administrator or give your [Powershell](https://docs.microsoft.com/en-us/powershell/scripting/overview?view=powershell-7.2)](https://docs.microsoft.com/en-us/powershell/scripting/overview?view=powershell-7.2) session enough privileges otherwise you will get an `"Access is denied"` error.

We recommend a [Powershell](https://docs.microsoft.com/en-us/powershell/scripting/overview?view=powershell-7.2) session with administrator privileges.

## Install the service

To install the SWS service use the `install` command along with a [configuration file](../configuration/config-file.md) for further SWS options customization.

Make sure to provide a configuration file to run SWS service properly. In particular, configure the server `address`, `port` and `root` directory accordingly.
If not so then the service might not start.

The following command will create the SWS service called `static-web-server` with a "`Static Web Server`" display name.

```powershell
static-web-server.exe -w C:\Users\MyUser\config.toml install
# Windows Service (static-web-server) is installed successfully!
# Start the service typing: sc.exe start "static-web-server" (it requires administrator privileges) or using the 'services.msc' application.
``` 

## Interact with the service

SWS doesn't provide a way to interact with Windows services directly. Instead, use the Windows built-in tools to interact with the SWS service once created.

For that purpose, you can use either the Windows [sc.exe](https://docs.microsoft.com/en-us/windows/win32/services/configuring-a-service-using-sc) or the [services.msc](https://docs.microsoft.com/en-us/windows/win32/services/services) application.

For example, using `sc.exe` you can show the SWS service configuration used once installed.

```powershell
sc.exe qc "static-web-server"
# [SC] QueryServiceConfig SUCCESS

# SERVICE_NAME: static-web-server
#         TYPE               : 10  WIN32_OWN_PROCESS
#         START_TYPE         : 3   DEMAND_START
#         ERROR_CONTROL      : 1   NORMAL
#         BINARY_PATH_NAME   : C:\Users\MyUser\static-web-server.exe 
#                                   --windows-service=true 
#                                   --config-file=C:\Users\MyUser\config.toml
#         LOAD_ORDER_GROUP   :
#         TAG                : 0
#         DISPLAY_NAME       : Static Web Server
#         DEPENDENCIES       :
#         SERVICE_START_NAME : LocalSystem
```

Remember that alternatively, you can also use the `services.msc` application if you prefer GUI service control.

### Start

To start the service use the following `sc.exe` command.

```powershell
sc.exe start "static-web-server"
# SERVICE_NAME: static-web-server
#     TYPE               : 10  WIN32_OWN_PROCESS
#     STATE              : 2  START_PENDING
#                             (NOT_STOPPABLE, NOT_PAUSABLE, IGNORES_SHUTDOWN)
#     WIN32_EXIT_CODE    : 0  (0x0)
#     SERVICE_EXIT_CODE  : 0  (0x0)
#     CHECKPOINT         : 0x0
#     WAIT_HINT          : 0x7d0
#     PID                : 3068
#     FLAGS              :
```

### Status

To show the service status use the following `sc.exe` command.

```powershell
sc.exe query "static-web-server"
# SERVICE_NAME: static-web-server
#     TYPE               : 10  WIN32_OWN_PROCESS
#     STATE              : 4  RUNNING
#                             (STOPPABLE, NOT_PAUSABLE, IGNORES_SHUTDOWN)
#     WIN32_EXIT_CODE    : 0  (0x0)
#     SERVICE_EXIT_CODE  : 0  (0x0)
#     CHECKPOINT         : 0x0
#     WAIT_HINT          : 0x0
```

### Stop

To stop the service use the following `sc.exe` command.

```powershell
sc.exe stop "static-web-server"
# SERVICE_NAME: static-web-server
#         TYPE               : 10  WIN32_OWN_PROCESS
#         STATE              : 3  STOP_PENDING
#                                 (STOPPABLE, NOT_PAUSABLE, IGNORES_SHUTDOWN)
#         WIN32_EXIT_CODE    : 0  (0x0)
#         SERVICE_EXIT_CODE  : 0  (0x0)
#         CHECKPOINT         : 0x2
#         WAIT_HINT          : 0xbb8
```

After stopping the service you can also show its status.

```powershell
sc.exe query "static-web-server"
# SERVICE_NAME: static-web-server
#         TYPE               : 10  WIN32_OWN_PROCESS
#         STATE              : 1  STOPPED
#         WIN32_EXIT_CODE    : 0  (0x0)
#         SERVICE_EXIT_CODE  : 0  (0x0)
#         CHECKPOINT         : 0x0
#         WAIT_HINT          : 0x0
```

## Uninstall the service

To uninstall the SWS service just use the `uninstall` command. Note that the service should be first stopped before uninstalling it.

```powershell
static-web-server.exe uninstall
# Windows Service (static-web-server) is uninstalled!
```

After uninstalling the service you can verify if removed.

```powershell
sc.exe qc "static-web-server"
# [SC] OpenService FAILED 1060:
#
# The specified service does not exist as an installed service.
```
