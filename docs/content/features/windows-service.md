# Windows Service

**`SWS`** can be also executed in a [Windows Service](https://docs.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2003/cc783643(v=ws.10)) context. Therefore it also provides a subcommand to *install* SWS as a Windows Service.

This feature is disabled by default and can be controlled by the boolean `-s, --windows-service` option or the equivalent [SERVER_WINDOWS_SERVICE](./../configuration/environment-variables.md#server_windows_service) env.

![Static Web Server running as a Windows Service](https://user-images.githubusercontent.com/1700322/169807572-d62a7bab-b596-4597-85f7-31a7c02aeefe.png)
> _Static Web Server running as a Windows Service and displayed by 'services.msc' application._

## Important Notes

- This is an obvious Windows platform-specific feature.
- The SWS Windows Service option (`windows-service`) doesn't create a Windows Service per se. It just tells SWS to run in a Windows Service context. So it's necessary to [install the SWS Windows Service](#install-the-service) first.
- Enabling the `windows-service` option via the [configuration file](../configuration/config-file.md) is unnecessary if you use the [install subcommand](#install-the-service) to create the service since it already enables it during the service installation.
- However, you can enable the `windows-service` option for example if you plan to create your own Windows Service and use SWS with it.

## Service privileges

To either install or uninstall the SWS Windows service requires *administrator* privileges, so make sure to open the terminal application as administrator or give your [Powershell](https://docs.microsoft.com/en-us/powershell/scripting/overview?view=powershell-7.2) session enough privileges otherwise you will get an `"Access is denied"` error.

We recommend a Powershell session with administrator privileges.

## Windows Firewall

You can serve content with SWS in a Windows network. However, if you face issues running SWS it could be due to missing firewall configuration. So you probably have to define an `inbound rule` to allow inbound network traffic on a specified TCP port of your choice.

Follow the steps below to adjust your firewall:

1. Configure an [Inbound Port Rule](https://docs.microsoft.com/en-us/windows/security/threat-protection/windows-firewall/create-an-inbound-port-rule) in your Windows firewall so clients can reach the server's port.
2. In your SWS config file, use the server IP as a host or a non-routable address like `0.0.0.0` if you prefer.
3. Create a Windows Service following https://static-web-server.net/features/windows-service/ and start it.
4. Finally, restart the service to apply the changes.

Note that the steps above are general and you have to adjust your firewall rule(s) according to your needs.

## Install the service

To install the SWS service use the `install` command along with a [configuration file](../configuration/config-file.md) for further SWS options customization.

Make sure to provide a configuration file to run SWS service properly. In particular, configure the server `address`, `port` and `root` directory accordingly.
If not then the service might not start.

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
