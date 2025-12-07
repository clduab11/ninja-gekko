# Debug Directory Structure

This directory contains all debug, log, and diagnostic text files for the ninja-gekko project. The structure enforces strict separation between current logs and historical/archive logs to minimize root-level file pollution and improve maintainability.

## Structure

- `current/`
  - Contains all active logs for the current development/debug sessions.
  - Example files:
    - `logs.txt`
    - `panic_logs.txt`

- `archive/2024-12-dev-checkpoints/`
  - Contains historical logs created at development checkpoints in December 2024.
  - All logs that are not relevant to the current session but are necessary for traceability, audit, or post-mortem analyses are kept here.
  - Example files:
    - `logs_after_cors_fix.txt`
    - `logs_after_fix.txt`
    - `logs_after_network_fix.txt`
    - `logs_clean_restart.txt`
    - `logs_final_check.txt`
    - `logs_final_final_check.txt`
    - `logs_final_final.txt`
    - `logs_final.txt`

## Conventions

- **Current session logs** should only be kept in `debug/current/` and rotated or archived periodically.
- **Historical/archive logs** must be moved into dated subfolders within `debug/archive/`, named based on the main development milestone or checkpoint.
- No `.txt` log or debug files should reside at the repository root.
- Periodically, all contents in `debug/current/` should be reviewed and shifted to an appropriate archive subfolder for long-term retention.

## Purpose

Centralizing debug and log data:
- Keeps the repository root clean
- Makes finding diagnostic artifacts fast and predictable
- Aids in onboarding, traceability, and knowledge transfer

Please keep this layout consistent for all future logs and debug artifacts.