# Seismometer project

Home earthquake notification service, Rust version.

# Configuration

This daemon requires quite a bit of customization through its configuration
file (and/or environment variables) to be useful. In general, configuration
is split into two main categories: Seismometer settings and MQTT settings.

## Seismometer settings (TBD)

## MQTT settings (TBD)

## Environment

Nearly all configuration items can be overridden from the environment.
To do so, one must set an environment variable named in such a way
that it will be picked up by this configuration builder. Use this
process to build up the correct name:

1. Start with the application prefix. In this case "`SEISMO`".
2. Append two underscores (i.e. "`__`").
3. Append the item or section name, in uppercase (e.g. "`MQTT`").
4. If further depth is required, go to step 2.

To set the MQTT password, for example, one would use the environment
variable name "`SEISMO__MQTT__PASSWORD`", which, in the Bourne shell would
be set with `export SEISMO__MQTT__PASSWORD=pass`.

# Rationale

There is an excellent existing project named "RS-UDP" which provides many
more features than this project, but it requires Python and can be a bit
heavy on resource-constrained machines and hard to get set up on somewhat
more "alternative" operating systems like FreeBSD.

This project re-imagines a part of RS-UDP to create a very lightweight MQTT
alarm system for local earthquake sensing. Being written in Rust, it can
be compiled into a very efficient and small binary that can run on very
resource-constrained servers.
