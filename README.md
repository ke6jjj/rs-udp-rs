# Seismometer project

Home earthquake notification service, Rust version.

## Rationale

There is an excellent existing project named "RS-UDP" which provides many
more features than this project, but it requires Python and can be a bit
heavy on resource-constrained machines and hard to get set up on somewhat
more "alternative" operating systems like FreeBSD.

This project re-imagines a part of RS-UDP to create a very lightweight MQTT
alarm system for local earthquake sensing. Being written in Rust, it can
be compiled into a very efficient and small binary that can run on very
resource-constrained servers.
