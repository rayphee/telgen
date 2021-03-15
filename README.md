# Telemetry Generator

Telemetry Generator (or `telgen` for short) is a command line utility that generates and logs endpoint activity in YAML. It can generate endpoint activity interactively or from lines in a properly formatted file.

## Dependencies

- `clap` == 2.27.0
- `chrono` == 0.4
- `whoami` == 1.1.1

This program was also tested on the following platforms:

- Linux (KDE Neon 5.21.1)
- Mac OS (Big Sur 11.1)

## Usage

To start an interactive telemetry generator session, execute the program without the input file command. By default, `telgen` will create `telemetry.log` in the working directory if it does not exist and _append_ to it if does; this can be changed via the `-l` option:

```console
$ ./telgen -l telemetry_2.log
Logging to telemetry_2.log
telgen> 
...
```

You can specify an input script with preformed lines by adding it as a command line argument. A test script is provided as an example named `testscript`:

```console
$ ./telgen SCRIPT
Logging to telemetry.log
...
```

### Starting a Process

To spawn a process, type the `SPAWN` keyword, followed by the executable's name/full path, and optionally arguments to be passed to the process:

```console
telgen> SPAWN ls -l
```

### Manipulating Files

To perform a file operation, type the `FILE` keyword, followed by the file operation (detailed below), full file path, and optionally ASCII data to append (for `MOD` operation only). The available `FILE` operations are:

- `NEW`: Create an empty file at the specified path

    ```console
    telgen> FILE NEW /home/user/test.txt
    ```

- `DEL`: Delete the file at the specified path

    ```console
    telgen> FILE DEL /home/user/test.txt
    ```

- `MOD`: Append the supplied data to the file located at the specified path

    ```console
    telgen> FILE MOD /home/user/test.txt data
    ```

### Establishing a Network Connection and Sending Data

To establish a network connection and send data, type the `NET` keyword, the source IP and port, followed by the destination IP address and port, and ASCII data:

```console
telgen> NET 127.0.0.1:1234 127.0.0.1:4321 data
```

`telgen` is designed to be robust, so any incorrect input or failed `telgen` commands will be reported to the user and ignored from execution and the telemetry log.

## Logged Data

Each endpoint activity is referenced by its timestamp. For all endpoint activity types, the following data is logged in their respective keys:

- Timestamp: `timestamp`
- Telgen command line argument: `command-line`
- Process ID of the executable: `pid`
- Name of the activity generating process: `process-name`
- Endpoint activity type: `activity-type`

For any `FILE` operation, the following additional information is logged in their respective keys:

- Type of `FILE` operation: `file-operation`
- Full path to target file: `file-path`

For any `NET` operation, the following additional information is logged in their respective keys:

- Originating IP address and port: `source`
- Destination IP address and port: `destination`
- Number of bytes sent to destination over the connection: `bytes-sent`
- Transmission protocol of connection: `protocol`
