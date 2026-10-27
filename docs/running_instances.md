# Running Instances

Sqrilizz provides full process tracking, console streaming, and live command management for both client and server instances.

## Launching

+++ App
Select an instance from the list on the Home page and click the `Launch` button in the action bar.
+++ CLI
Run `sqrilizz launch <instance>` to start an instance.
+++

## Commanding Servers

With server instances, you can send console commands to manage the server in real time.

+++ App
Navigate to the instance console and send commands using the input bar at the bottom. Press enter to submit.
+++ CLI
In the original terminal where the instance was launched, type and press enter to send commands.
+++

## Background Execution

When an instance is launched, Sqrilizz manages the game process in the background and streams logs.

+++ App
You can keep Sqrilizz open to monitor real-time memory usage and console logs, or minimize the window while playing.
+++ CLI
The CLI will stream stdout/stderr until the process finishes or until you detach.
+++
