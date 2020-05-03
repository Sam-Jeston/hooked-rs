# Hooked-rs

A utility for accepting GitHub webhooks and running a set of commands.

This application was developed to easily facilitate updating small side-projects on a VPS from GitHub webhooks. The base memory of the application if 500kbs, making it perfect for runnings on a VPS with a small memory limit.

## Getting Started

1. Build the application from source (with the rust nightly toolchain): `cargo build --release`. The default output binary path is `./target/release/hooked-rs`
2. Configure the application via a yaml file. `example.yml` can be used for reference.
3. Run it - `./hooked-rs --config path-to-config.yml`

## YAML Configuration
Properties:
 - port [int]: The port for the hook API to listen on.
 - host [string]: The host namespace.
 - log [string]: The path to the log output of the application.
 - targets [Array<Target>]: A list of target applications

Target type:
 - repository
 - branch
 - directory
 - steps [Array<string>]: Steps to run when a hook is received. The only step that occurs by default is changing directory to the one provided for the target. All git commands to update the source directory must be specified.

See example.yml for a functional example.

## Limitations

1. hooked-rs does not manage GitHub authentication.
2. hooked-rs does not persist any state. If there are jobs in the queue and the application is terminated, these jobs will be lost. However a job cannot panic the main thread, so this is unlikely to happen.
3. The shell environment is currently hardcoded to `sh`, making this unsuitable for Windows. This may be made configurable in the future. PRs welcome.

## HTTP status codes
 - 200: Status hook received and added to queue for a known target and branch
 - 202: Status hook received for a known target, but the state of the hook was not "success", so no action was taken
 - 204: Status hook received for a known target, but a branch did not match, so no action was taken
 - 404: Status hook received but it did not match any target 
 - 422: Unhandled webhook. All webhook types except "status" will return this. To prevent this, configure your webhook to only send status hooks.

As target jobs are run asyncronously, the HTTP status code returned to GH will not reflect the success of the underlying job.
